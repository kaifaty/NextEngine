# Physical World Layer
## Единая архитектура физической системы мира для игрового движка на Rust

**Статус:** umbrella research / architecture brief
**Контекст:** август 2026 года
**Назначение:** исходный документ для последующей генерации ADR, технических спецификаций, research spikes, milestones и validation plans.

Связанные документы:

- `continuum_physics_water_mud_offroad_research_brief.md`
- `vegetation_physics_destructible_trees_research.md`
- `vegetation_physics_followup_production_architecture.md`

Этот документ не заменяет их. Он описывает **единый физический слой мира**, в который должны войти разработанные ранее подсистемы воды, почвы, грязи, деревьев, ветра, огня, rigid bodies, articulated bodies и их взаимодействия.

---

# 0. Краткий вывод

Да, исследованные системы следует объединить в одну архитектуру верхнего уровня:

```text
PhysicalWorld
```

Но это **не означает один универсальный solver**.

Правильная формулировка:

> **Один физический мир — несколько специализированных solvers.**

Каждый физический домен сохраняет собственную математику:

- rigid bodies — rigid-body solver;
- персонажи и механизмы — articulated-body solver;
- вода, грязь, песок, снег и почва — continuum solver;
- деревья и другие растения — rods/beams/modal structural solver;
- ветер и осадки — atmosphere/weather field;
- тепло, возгорание и фазовые переходы — thermal/fire solver;
- разрушение — material-specific damage/fracture models.

Объединение происходит на уровне:

- общей системы координат и времени;
- источников истины;
- coupling contracts;
- обмена импульсом, массой, влагой и теплом;
- Representation Promotion / Physics LOD;
- streaming;
- persistence;
- replication;
- semantic queries;
- validation;
- profiling.

Центральная архитектурная формулировка:

> **PhysicalWorld — оркестратор специализированных физических доменов, объединённых общей системой материалов, coupling-протоколами, representation promotion, persistence и семантическими queries.**

---

# 1. Зачем нужен единый физический слой

Без единого слоя каждая система будет реализовывать свои изолированные игровые реакции:

```text
машина вошла в грязь
→ traction *= 0.5

дерево намокло
→ can_burn = false

дерево ударили
→ tree_hp -= damage

вода коснулась огня
→ fire.delete()
```

Такая архитектура плохо масштабируется:

- комбинации эффектов приходится прописывать вручную;
- появляются противоречащие друг другу источники состояния;
- невозможно физически корректно передавать энергию и импульс;
- AI видит набор специальных флагов вместо состояния мира;
- каждый новый материал требует большого числа исключений;
- persistence и multiplayer усложняются экспоненциально.

В целевой архитектуре системы обмениваются физическими величинами:

```text
масса
импульс
сила
момент
тепло
влага
давление
напряжение
повреждение
химическая энергия
```

Пример:

```text
дождь
→ передал массу воды почве
→ saturation увеличилась
→ pore pressure изменилось
→ effective soil strength уменьшилась
→ корни удерживаются хуже
→ дерево стало менее устойчивым
```

Ни одна подсистема не должна знать весь сценарий целиком.

---

# 2. Главные архитектурные принципы

## 2.1. Один мир, несколько solvers

Не пытаться объединить:

```text
rigid bodies
+
Navier–Stokes
+
Cosserat rods
+
combustion
+
soil plasticity
```

в одну глобальную систему уравнений.

Для игрового движка это будет:

- чрезмерно дорого;
- трудно отлаживать;
- плохо совместимо с LOD;
- плохо совместимо со streaming;
- сложно распараллеливать;
- сложно сохранять;
- сложно реплицировать по сети.

Общая архитектура должна быть универсальной на уровне **протоколов взаимодействия**, а не на уровне одной математики.

---

## 2.2. Один источник истины для каждого типа состояния

Не дублировать физическое состояние между доменами.

Примеры:

| Состояние | Владелец |
|---|---|
| Положение автомобиля | `RigidDomain` |
| Положение суставов NPC | `ArticulatedDomain` |
| Масса и скорость воды | `ContinuumDomain` |
| Saturation и pore pressure почвы | `ContinuumDomain` |
| Структурное повреждение дерева | `VegetationDomain` |
| Температура сегмента дерева | один явно выбранный владелец |
| Глобальный ветер | `AtmosphereDomain` |
| Persistent cut дерева | `VegetationPersistence` |

Coupler может читать состояние и передавать воздействия, но не должен создавать второй источник истины.

---

## 2.3. Couplers вместо прямого доступа

Плохо:

```rust
vegetation.tree[id].root_strength *=
    continuum.soil[cell].wetness;
```

Лучше:

```text
VegetationSoilCoupler
    ├── читает root contact region
    ├── запрашивает soil support state
    ├── вычисляет distributed reaction
    └── передаёт симметричные воздействия
```

Домены не должны напрямую изменять внутреннее состояние друг друга.

---

## 2.4. Representation Promotion — часть физики мира

Каждый домен может иметь несколько представлений одного объекта или региона.

Примеры:

```text
дерево:
shader → modal → rods → local fibers

вода:
spectral → shallow/reduced → particles → refined multiphase

почва:
persistent reduced state → active particles → refined wheel-contact region

огонь:
burn summary → segment thermal → radial burn → local volumetric
```

LOD здесь не просто визуальная оптимизация.

Он определяет, **какой физической моделью сейчас представлен объект**.

---

## 2.5. Компактное persistent state вместо хранения всех DOF

Не хранить:

- все water particles для спящего региона;
- все rod nodes каждого дерева;
- все thermal voxels;
- все soil particles открытого мира.

Хранить:

- компактное физически значимое состояние;
- topology changes;
- пластическую историю в агрегированном виде;
- cuts;
- missing branches;
- saturation;
- rut state;
- burn state.

При активации подробное представление реконструируется.

---

## 2.6. Семантические queries поверх solver internals

Gameplay и AI не должны знать:

- SPH multiplier;
- rod constraint lambda;
- pressure iteration count;
- deformation-gradient storage layout.

Они должны получать:

```text
глубина воды
скорость течения
несущая способность
риск проваливания
структурная устойчивость дерева
температура
риск возгорания
направление падения
```

---

## 2.7. Physics events не заменяют состояние

События нужны для:

- AI;
- gameplay;
- VFX;
- audio;
- networking;
- analytics.

Но событие:

```text
TreeIgnited
```

не заменяет состояние:

```text
temperature
moisture
char depth
fuel mass
```

---

## 2.8. Production path и research path должны сосуществовать

Для каждого рискованного направления нужен зрелый baseline.

Примеры:

```text
production water:
GPU DFSPH

research water:
variational + DDFK hybrid
```

```text
production tree:
modal + coarse rods + section damage

research tree:
strand/fiber fracture
```

```text
production soil:
single-phase granular/viscoplastic model

research soil:
two-phase pore-water poromechanics
```

Research-функции не должны блокировать работоспособный движок.

---

# 3. Высокоуровневая структура

```text
                          GAMEPLAY / AI
                                │
                  queries, commands, events
                                │
                                ▼
                      ┌──────────────────┐
                      │  PHYSICAL WORLD  │
                      └──────────────────┘
                                │
       ┌────────────────────────┼────────────────────────┐
       │                        │                        │
 Simulation Scheduler     Coupling Graph      Representation Manager
       │                        │                        │
       │                        │                        │
       ▼                        ▼                        ▼
┌───────────────┐    exchange of impulse,      Physics LOD / streaming
│ Domain Solvers│    mass, heat, moisture,
└───────────────┘    pressure and topology
       │
       ├──────────────┬──────────────┬──────────────┐
       ▼              ▼              ▼              ▼
   Rigid          Articulated     Continuum      Vegetation
       │              │              │              │
       ├──────────────┼──────────────┼──────────────┤
       ▼              ▼              ▼              ▼
 Atmosphere        Thermal        Fracture      Persistence
```

---

# 4. Предлагаемые верхнеуровневые модули

```text
PhysicalWorld
│
├── WorldPhysicsCore
├── SimulationScheduler
├── CouplingGraph
├── RepresentationManager
├── MaterialRegistry
├── SpatialInteractionIndex
│
├── RigidDomain
├── ArticulatedDomain
├── ContinuumDomain
├── VegetationDomain
├── AtmosphereDomain
├── ThermalDomain
├── FractureServices
│
├── WorldPhysicsQueries
├── WorldPhysicsEvents
├── WorldPhysicsPersistence
├── WorldPhysicsReplication
├── WorldPhysicsValidation
└── WorldPhysicsDebug
```

---

# 5. `WorldPhysicsCore`

Отвечает за общие понятия, но не содержит конкретную физическую модель.

Включает:

- идентификаторы объектов;
- идентификаторы регионов;
- единицы измерения;
- систему времени;
- координатные пространства;
- общие типы воздействий;
- contracts владения состоянием;
- common handles;
- lifecycle topology events.

Пример:

```rust
pub struct PhysicsEntityId(u64);
pub struct PhysicsRegionId(u64);
pub struct MaterialId(u32);
pub struct DomainId(u16);
```

Не создавать один огромный enum всех объектов мира.

---

# 6. `RigidDomain`

## 6.1. Ответственность

Источник истины для:

- rigid-body pose;
- linear velocity;
- angular velocity;
- mass;
- inertia;
- contacts;
- sleeping;
- constraints между rigid bodies.

Типичные объекты:

- автомобили;
- камни;
- оружие;
- лодки;
- ящики;
- двери;
- упавшие брёвна после упрощения;
- detached debris.

---

## 6.2. Взаимодействия

```text
Rigid ↔ Continuum
Rigid ↔ Vegetation
Rigid ↔ Articulated
Rigid ↔ Thermal
Rigid ↔ Fracture
```

Примеры:

- лодка получает pressure impulses от воды;
- колесо деформирует грязь;
- автомобиль ударяет дерево;
- нагретый металл передаёт тепло;
- разрушенная ветка становится rigid body.

---

## 6.3. Domain API

Концептуально:

```rust
pub struct RigidBodySnapshot {
    pub transform: Transform,
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
    pub mass: f32,
    pub inertia: Mat3,
}

pub struct RigidImpulse {
    pub body: PhysicsEntityId,
    pub world_point: Vec3,
    pub impulse: Vec3,
}
```

Couplers должны работать через batched interfaces.

---

# 7. `ArticulatedDomain`

## 7.1. Ответственность

Источник истины для:

- articulated body graph;
- joints;
- joint limits;
- motors;
- contacts;
- joint impulses;
- actuators.

Типичные объекты:

- NPC;
- игрок;
- животные;
- роботы;
- физические механизмы;
- подвеска и части транспорта, если не включены в rigid solver.

---

## 7.2. Связь с Motor System

```text
Strategic Agent
→ Tactical Controller
→ Skill / Motion Policy
→ Low-level Motor Policy
→ joint targets / torques
→ ArticulatedDomain
→ contacts with PhysicalWorld
```

Articulated solver должен взаимодействовать не только с rigid surfaces, но и с:

- водой;
- грязью;
- песком;
- снегом;
- ветвями;
- обломками;
- течением;
- огнём и температурой.

---

## 7.3. Семантические outputs

Для Motor System полезны:

```text
support forces
foot sinkage
surface velocity
material compliance
contact stability
water drag
branch deflection
```

---

# 8. `ContinuumDomain`

## 8.1. Ответственность

Объединяет active high-fidelity continuum materials:

```text
water
mud
soil
sand
snow
sediment
slurry
wet clay
granular materials
```

---

## 8.2. Физическое представление

Основной исследованный путь:

```text
adaptive Lagrangian material particles
+
material-specific constitutive laws
+
neighbor graph
+
variational / constraint solver
```

Постоянная world-space grid не является фундаментальным физическим состоянием.

Временные grid/hash структуры разрешены для:

- neighbor search;
- rendering;
- reduced LOD;
- storage;
- pressure acceleration;
- sleeping-region representation.

---

## 8.3. Внутренние solver backends

```text
Fluid:
- DFSPH baseline
- PBF/IPBF
- variational research
- DDFK hybrid research

Granular/soil:
- Drucker–Prager
- μ(I)
- Cam-Clay / Modified Cam-Clay
- viscoplastic models

Multiphase:
- soil ↔ water
- sediment ↔ water
- local air ↔ water
```

---

## 8.4. Состояние

Примеры material-specific state:

```text
position
velocity
mass
rest volume
density
stress
deformation gradient
plastic state
porosity
saturation
pore pressure
temperature
phase
```

Не хранить все поля в одном `UniversalParticle`.

Использовать специализированные SoA storage layouts.

---

## 8.5. Взаимодействия

```text
Continuum ↔ Rigid
Continuum ↔ Articulated
Continuum ↔ VegetationRoots
Continuum ↔ Atmosphere
Continuum ↔ Thermal
Continuum ↔ Fracture/erosion
```

---

# 9. `VegetationDomain`

## 9.1. Ответственность

Источник истины для:

- структурного графа дерева;
- modal state;
- rod/beam state;
- branch loads;
- wood damage;
- cuts;
- fracture;
- foliage anchors;
- moisture;
- burn state;
- decay;
- root state.

---

## 9.2. Представления дерева

```text
StaticTree
→ ShaderTree
→ ModalTree
→ StructuralTree
→ RefinedFractureTree
```

Detached lifecycle:

```text
DetachedStructural
→ DynamicCompound
→ SleepingRigid
→ StaticDebris
```

---

## 9.3. Основная механика

```text
trunk + major branches
→ hierarchical Cosserat rods / beams

cut/fracture region
→ refined cross-section / strands / fibers

leaves
→ foliage clusters + procedural flutter

roots
→ coarse structural graph + soil coupling
```

---

## 9.4. Взаимодействия

```text
Vegetation ↔ Atmosphere
Vegetation ↔ ContinuumSoil
Vegetation ↔ Rigid
Vegetation ↔ Articulated
Vegetation ↔ Thermal
Vegetation ↔ Fire spread
```

---

# 10. `AtmosphereDomain`

## 10.1. Первая версия

Не полноценная CFD-атмосфера.

Содержит поля:

```text
mean wind
gusts
turbulence
temperature
humidity
rain flux
snow flux
```

API:

```rust
pub struct AtmosphereSample {
    pub velocity: Vec3,
    pub temperature: f32,
    pub humidity: f32,
    pub rain_flux: f32,
    pub snow_flux: f32,
}
```

---

## 10.2. Потребители

- vegetation aerodynamic loads;
- embers;
- smoke;
- evaporation;
- rainfall;
- snow accumulation;
- vehicle aerodynamics;
- character wind forces;
- surface drying.

---

## 10.3. Будущие уровни

```text
Level 0:
global weather vectors

Level 1:
procedural spatial fields

Level 2:
terrain/building modifiers

Level 3:
local CFD regions for hero effects
```

---

# 11. `ThermalDomain`

## 11.1. Ответственность

Общие процессы энергии:

```text
temperature
heat transfer
conduction
convection approximation
radiation approximation
evaporation
melting
freezing
ignition
pyrolysis
```

---

## 11.2. Material-specific response

ThermalDomain не должен сам решать всё поведение материала.

Примеры:

```text
wood receives heat
→ Vegetation material updates moisture/char/strength

water receives heat
→ Continuum updates phase/evaporation

snow receives heat
→ Continuum converts snow to water

soil dries
→ Continuum changes saturation and strength
```

---

## 11.3. Ownership decision

Для каждого типа thermal state нужно заранее определить владельца.

Вариант A:

```text
ThermalDomain owns all temperature state
```

Вариант B:

```text
each domain owns its temperature
ThermalDomain only transfers heat
```

Для первого этапа предпочтительнее B:

> Домен владеет material state, ThermalDomain владеет coupling и общими heat-transfer algorithms.

---

# 12. `FractureServices`

Не один универсальный fracture solver.

Общие сервисы:

- damage lifecycle;
- topology split protocol;
- detached object creation;
- debris registration;
- persistent fracture record;
- fracture events;
- render fracture descriptors.

Material-specific models:

```text
wood:
anisotropic fibers

rock:
brittle fracture

metal:
plasticity + ductile failure

soil:
yield / plastic flow

ice:
brittle fracture
```

---

# 13. `CouplingGraph`

Это центральный механизм единого физического мира.

```text
                         CouplingGraph

       Rigid ◄──────────────► Continuum
         │                       │
         │                       │
         ▼                       ▼
    Vegetation ◄───────────► Atmosphere
         │                       │
         └────────────► Thermal ◄┘
```

Каждое ребро графа является отдельным coupler module.

---

# 14. Базовые couplers

```text
RigidContinuumCoupler
ArticulatedContinuumCoupler

RigidVegetationCoupler
ArticulatedVegetationCoupler

VegetationAtmosphereCoupler
VegetationSoilCoupler

AtmosphereContinuumCoupler
AtmosphereThermalCoupler

ThermalVegetationCoupler
ThermalContinuumCoupler
ThermalRigidCoupler

FireSpreadCoupler
```

---

# 15. Coupling quantities

Стандартизировать типы обмена.

## 15.1. Momentum

```rust
pub struct MomentumExchange {
    pub source: PhysicsEntityId,
    pub target: PhysicsEntityId,
    pub world_point: Vec3,
    pub impulse: Vec3,
}
```

## 15.2. Heat

```rust
pub struct HeatExchange {
    pub source: PhysicsEntityId,
    pub target: PhysicsEntityId,
    pub energy_joules: f32,
}
```

## 15.3. Mass

```rust
pub struct MassTransfer {
    pub material: MaterialId,
    pub source: PhysicsEntityId,
    pub target: PhysicsEntityId,
    pub mass_kg: f32,
}
```

## 15.4. Moisture

Может быть частным случаем mass transfer воды.

## 15.5. Topology

```rust
pub enum TopologyChange {
    Split,
    Detach,
    Merge,
    Erode,
    Deposit,
    Uproot,
}
```

---

# 16. Симметрия и conservation

Couplers должны по возможности сохранять:

- массу;
- линейный импульс;
- угловой импульс;
- энергию с понятной диссипацией.

Пример rigid-fluid:

\[
\Delta p_{rigid} = -\Delta p_{fluid}
\]

Пример torque:

\[
\Delta L = (x-x_{COM}) \times \Delta p
\]

Нельзя передавать силу только одной стороне.

---

# 17. Explicit vs iterative coupling

## 17.1. Explicit

Один обмен за timestep.

Подходит для:

- тяжёлого объекта в воде;
- слабой нагрузки ветра;
- rainfall;
- медленного thermal transfer.

## 17.2. Iterative

Повторный обмен внутри substep.

Нужен для:

- лёгких плавающих тел;
- deformable tire ↔ mud;
- root ↔ soft soil;
- падающего дерева в воду;
- сильного articulated-fluid interaction.

Пример:

```text
for coupling_iteration:
    continuum predicts pressure impulse
    rigid updates pose
    continuum updates boundary
```

---

# 18. Coupling stability classes

Каждый coupler должен объявлять:

```text
weak
moderate
strong
```

И требования:

- update order;
- substeps;
- iteration count;
- convergence metric;
- fallback.

---

# 19. `RepresentationManager`

Один из ключевых компонентов.

Отвечает за:

- выбор физического представления;
- relevance scoring;
- promotion;
- demotion;
- state transfer;
- GPU pool allocation;
- streaming;
- sleeping;
- predictive activation;
- persistent reconstruction.

---

# 20. Representation ladders

## 20.1. Water

```text
SpectralOcean
→ ReducedSurface/Flow
→ ParticleContinuum
→ RefinedMultiphaseRegion
```

## 20.2. Soil

```text
PersistentTerrainState
→ ActiveContinuumRegion
→ RefinedContactRegion
```

## 20.3. Trees

```text
Static
→ Shader
→ Modal
→ Structural
→ RefinedFracture
```

## 20.4. Fire

```text
FuelSummary
→ SegmentThermal
→ RadialBurn
→ LocalVolumetric
```

---

# 21. Relevance scoring

Использовать не только distance.

Общие факторы:

```text
camera distance
visibility
player proximity
vehicle trajectory
AI relevance
current damage
fire
high stress
future collision probability
network authority
quest relevance
```

Пример:

```text
дерево далеко, но горит
→ высокий thermal relevance

почва вне кадра, но под движущимся автомобилем
→ высокий mechanical relevance

вода рядом, но спокойная и закрытая
→ низкий refinement relevance
```

---

# 22. Predictive activation

Promotion должен происходить до события.

Примеры:

```text
tree safety factor падает
→ activate structural rods before fracture
```

```text
vehicle trajectory intersects soft terrain
→ activate continuum region before wheel contact
```

```text
fast object approaches water
→ refine impact region before splash
```

---

# 23. State transfer

Каждая лестница представлений должна иметь формализованный transfer.

## 23.1. Tree modal → rods

Передать:

- shape;
- velocity;
- approximate strain;
- damage;
- cuts;
- thermal state.

## 23.2. Soil persistent → particles

Реконструировать:

- surface;
- compaction;
- saturation;
- plastic history;
- displaced mass.

## 23.3. Water reduced → particles

Передать:

- mass;
- momentum;
- surface;
- boundary conditions.

## 23.4. Particles → persistent

Сжать:

- total material mass;
- surface/depth;
- momentum summary;
- saturation;
- plastic deformation.

---

# 24. `MaterialRegistry`

Общая система материалов должна описывать capabilities.

Не использовать один огромный `Material` со всеми возможными полями.

Концептуально:

```rust
pub struct MaterialDefinition {
    pub mechanical: Option<MechanicalMaterial>,
    pub fluid: Option<FluidMaterial>,
    pub thermal: Option<ThermalMaterial>,
    pub porous: Option<PorousMaterial>,
    pub fracture: Option<FractureMaterial>,
    pub combustion: Option<CombustionMaterial>,
}
```

---

# 25. Материальные capabilities

## 25.1. Mechanical

```text
density
elasticity
plasticity
damping
friction
```

## 25.2. Fluid

```text
viscosity
rest density
surface tension
yield stress
```

## 25.3. Porous

```text
porosity
permeability
saturation curve
capillary pressure
```

## 25.4. Thermal

```text
heat capacity
conductivity
phase thresholds
```

## 25.5. Fracture

```text
strength
toughness
anisotropy
damage law
```

## 25.6. Combustion

```text
ignition
pyrolysis
char yield
heat release
```

---

# 26. Примеры материалов

```text
Water
├── fluid
└── thermal

Wood
├── mechanical
├── fracture
├── thermal
├── porous/moisture
└── combustion

Wet Soil
├── mechanical
├── porous
├── fluid coupling
└── thermal

Steel
├── mechanical
├── plasticity
├── fracture
└── thermal

Snow
├── mechanical
├── plasticity
├── thermal
└── phase transition
```

---

# 27. Material state vs material definition

`MaterialDefinition` — неизменяемые параметры типа материала.

`MaterialState` — текущее состояние конкретного объекта/региона.

Пример дерева:

```text
definition:
oak wood properties

state:
moisture = 0.32
temperature = 410 K
damage = 0.21
char_depth = 4 mm
```

---

# 28. `SimulationScheduler`

PhysicalWorld должен поддерживать multi-rate simulation.

Разные домены обновляются с разной частотой.

Пример:

```text
Rigid:
120 Hz

Articulated:
60–120 Hz

Active continuum:
30–60 Hz + substeps

Structural trees:
30–60 Hz

Modal trees:
15–30 Hz

Thermal/fire:
5–20 Hz

Weather:
1–10 Hz
```

---

# 29. Dependency graph

Scheduler должен строить зависимости:

```text
Atmosphere
    ↓
Vegetation wind loads
    ↓
Structural solve

Continuum
    ↔
Rigid

Thermal
    ↓
Material property update
    ↓
Fracture
```

Нельзя полагаться на неявный порядок systems.

---

# 30. Пример общего timestep

```text
1. Stream regions and representations
2. Update relevance scores
3. Promote/demote representations
4. Update atmosphere/weather fields
5. Collect external commands
6. Predict domain states
7. Build cross-domain interaction candidates
8. Run weak couplers
9. Run strong coupling iterations
10. Solve internal domain constraints
11. Apply heat/mass/moisture transfers
12. Commit positions and velocities
13. Evaluate material damage/yield
14. Process topology changes
15. Update persistent state
16. Produce semantic events
17. Publish queries
18. Record debug/validation metrics
```

Точный порядок должен быть определён ADR и spec.

---

# 31. Topology changes

Особая категория событий:

```text
tree branch detached
tree uprooted
water region split
material eroded
soil deposited
rigid object fractured
snow melted
```

Topology changes должны быть:

- authoritative;
- persistable;
- replayable;
- networkable;
- ordered.

---

# 32. Transactional topology stage

Рекомендуется не изменять topology посреди произвольного solver pass.

Собирать requests:

```text
PendingTopologyChanges
```

Применять в выделенной стадии:

```text
end of substep / safe point
```

Это снижает риск invalid handles и race conditions.

---

# 33. `SpatialInteractionIndex`

Нужен общий broadphase верхнего уровня.

Он не заменяет внутренние neighbor structures доменов.

Назначение:

```text
какие домены/объекты потенциально взаимодействуют
```

Примеры:

- vehicle AABB intersects active soil region;
- tree roots overlap saturated region;
- burning branch near dry foliage;
- articulated foot enters water volume.

---

# 34. Domain-local spatial structures

Каждый домен сохраняет собственную структуру:

```text
Rigid:
BVH / broadphase

Continuum:
spatial hash / Morton sort

Vegetation:
tree BVH + rod collision

Atmosphere:
field chunks

Thermal:
active heat regions
```

Общий индекс только связывает домены.

---

# 35. Persistence

Мир должен сохранять физические последствия.

Примеры:

- колеи;
- displaced soil;
- puddles;
- saturation;
- cuts;
- missing branches;
- fallen trees;
- char;
- burned area;
- root damage.

---

# 36. Delta persistence

Pristine state генерируется из world seed/asset.

Сохраняются только изменения.

```text
untouched tree
→ no save record

cut tree
→ cut delta

deformed terrain
→ region delta

burned area
→ thermal/material delta
```

---

# 37. Persistent region state

Концептуально:

```rust
pub struct PhysicalRegionState {
    pub material_deltas: Vec<MaterialDelta>,
    pub topology_deltas: Vec<TopologyDelta>,
    pub thermal_deltas: Vec<ThermalDelta>,
    pub vegetation_deltas: Vec<VegetationDelta>,
}
```

Реальная реализация должна быть компактной и chunk-based.

---

# 38. Sleeping continuum region

Нельзя сохранять все particles.

Переход:

```text
active particles
    ↓ homogenization/compression
persistent reduced state
    ↓ later activation
reconstructed particles
```

Хранить:

- material mass;
- terrain surface;
- layer composition;
- compaction;
- saturation;
- plastic strain summary;
- residual flow if important.

---

# 39. Sleeping vegetation state

Хранить:

```text
cuts
damage
missing branches
lean
burn state
moisture
decay
root damage
fallen transform
```

Не хранить rod positions для каждого дерева.

---

# 40. Networking and replication

Полная репликация solver DOFs невозможна.

Основной принцип:

> Сервер authoritative для topology и gameplay-relevant state; клиенты локально воспроизводят вторичную динамику.

---

# 41. Реплицируемые события

```text
TreeCut
TreeFracture
TreeUproot
MaterialRegionActivated
TerrainDeformedSummary
IgnitionStarted
MaterialTransferred
RigidObjectEnteredFluid
```

---

# 42. Нереплицируемые детали

Можно оставить локальными:

- точное колебание листьев;
- мелкие water ripples;
- вторичные брызги;
- мелкие ветки;
- smoke turbulence;
- небольшие rod differences.

---

# 43. Correction snapshots

Для важных активных объектов можно отправлять редкие coarse snapshots:

```text
tree trunk pose
vehicle pose
water region momentum summary
terrain surface delta
```

---

# 44. Determinism

Полная bitwise determinism GPU solvers может быть неоправданной.

Разделить:

```text
authoritative discrete state:
deterministic

continuous cosmetic state:
approximately reproducible
```

---

# 45. WorldPhysics Queries

Gameplay и AI используют агрегированные запросы.

---

# 46. Traversability query

```rust
pub struct TraversabilitySample {
    pub support_strength: f32,
    pub sinkage_risk: f32,
    pub slip_risk: f32,

    pub water_depth: f32,
    pub flow_velocity: Vec3,

    pub surface_velocity: Vec3,
    pub temperature: f32,

    pub fire_risk: f32,
}
```

---

# 47. Structural hazard query

```rust
pub struct StructuralHazard {
    pub unstable: bool,
    pub stability: f32,

    pub fall_direction: Vec3,
    pub estimated_failure_time: Option<f32>,

    pub burning: bool,
}
```

---

# 48. Material sample query

```rust
pub struct MaterialSample {
    pub material: MaterialId,

    pub density: f32,
    pub velocity: Vec3,
    pub pressure: f32,

    pub temperature: f32,

    pub saturation: f32,
    pub yield_state: f32,
}
```

---

# 49. Query LOD

AI не требует максимальной точности.

Можно иметь:

```text
coarse strategic map
medium tactical query
high-detail local motor query
```

Пример:

```text
Strategic:
болото непроходимо

Tactical:
левая сторона суше

Motor:
конкретная опора стопы/колеса
```

---

# 50. Semantic event layer

Примеры событий:

```text
ContactStarted
ContactEnded

MaterialYielded
MaterialFlowStarted
MaterialRegionFlooded

StructureDamaged
FractureStarted
StructureDetached
TreeFalling
RootFailure

IgnitionStarted
CombustionIntensified
Extinguished

VehicleTractionLost
VehicleHighCentered
```

---

# 51. Events для AI

AI может реагировать:

```text
TreeFalling
→ evade

RegionFlooded
→ replan path

VehicleTractionLost
→ reduce throttle / engage lock

IgnitionStarted
→ flee / extinguish / exploit

RootFailure
→ danger zone
```

---

# 52. Events для audio/VFX

Физический слой публикует физические descriptors:

```text
impact energy
material pair
fracture size
fluid displacement
heat release
```

Audio/VFX выбирают представление.

---

# 53. Debugging

Общий debug layer должен визуализировать cross-domain state.

---

# 54. Общие debug overlays

```text
domain ownership
representation state
coupling edges
active regions
sleeping regions
topology changes
heat transfers
mass transfers
impulses
```

---

# 55. Continuum overlays

```text
density error
divergence
pressure
velocity
saturation
pore pressure
plastic strain
particle level
```

---

# 56. Vegetation overlays

```text
modal/rod/fiber state
stress
curvature
damage
cuts
safety factor
root forces
temperature
char depth
```

---

# 57. Scheduler overlays

```text
update frequency
substeps
coupling iterations
dependency graph
GPU queues
synchronization points
```

---

# 58. Validation

Нужны domain tests и integrated multiphysics tests.

---

# 59. Domain tests

См. связанные документы:

- water/soil benchmarks;
- beam/fracture/tree benchmarks;
- rigid-body benchmarks;
- thermal benchmarks.

---

# 60. Integrated tests

## 60.1. Vehicle in wet soil

```text
rain
→ saturation
→ wheel contact
→ rut
→ slip
```

Проверить:

- mass conservation;
- soil displacement;
- vehicle reaction;
- traction curve.

---

## 60.2. Tree uprooting

```text
wet soil
+
wind
+
root forces
→ uprooting
```

Проверить:

- reaction symmetry;
- soil plastic work;
- root failure;
- tree momentum.

---

## 60.3. Tree falling into water

```text
tree fracture
→ falling rods
→ water impact
→ splash
→ floating/sinking trunk
```

Проверить:

- impulse exchange;
- stability;
- topology transition;
- structural → rigid conversion.

---

## 60.4. Fire after rain

```text
rain
→ wood moisture
→ ignition attempt
→ slower heating
```

Проверить:

- water mass transfer;
- thermal energy;
- ignition delay;
- material state consistency.

---

## 60.5. Burning tree collapse

```text
combustion
→ char depth
→ section strength loss
→ fracture
```

Проверить:

- thermal/material coupling;
- failure timing;
- topology event.

---

# 61. Cross-domain conservation metrics

Отслеживать:

```text
total mass by material
total linear momentum
total angular momentum
total thermal energy
known dissipated energy
topology object count
```

Для открытых систем учитывать:

- rain;
- evaporation;
- boundaries;
- external forces;
- combustion mass release.

---

# 62. Performance metrics

```text
active entities per domain
active particles
active rods
modal trees
refined regions
coupling pairs
coupling iterations
solver time
promotion time
state transfer time
GPU memory
streaming bandwidth
persistence size
network event rate
```

---

# 63. Rust project structure

Не обязательно создавать все crates сразу.

Целевое разбиение:

```text
crates/
├── world-physics-core
│   ├── ids
│   ├── units
│   ├── time
│   ├── ownership
│   ├── common_types
│   └── topology
│
├── world-physics-schedule
│   ├── multi_rate
│   ├── substeps
│   ├── dependency_graph
│   └── gpu_schedule
│
├── world-physics-coupling
│   ├── graph
│   ├── momentum
│   ├── heat
│   ├── mass
│   ├── moisture
│   └── strong_coupling
│
├── world-physics-representation
│   ├── relevance
│   ├── promotion
│   ├── demotion
│   ├── state_transfer
│   ├── streaming
│   └── sleeping
│
├── world-physics-materials
│   ├── mechanical
│   ├── fluid
│   ├── porous
│   ├── thermal
│   ├── fracture
│   └── combustion
│
├── physics-rigid
├── physics-articulated
├── physics-continuum
├── physics-vegetation
├── physics-atmosphere
├── physics-thermal
│
├── world-physics-query
├── world-physics-events
├── world-physics-persistence
├── world-physics-replication
├── world-physics-validation
└── world-physics-debug
```

Для первого этапа лучше:

```text
world_physics/
├── core
├── schedule
├── coupling
├── representation
├── domains
├── query
├── persistence
└── debug
```

---

# 64. Domain interface

Концептуальный интерфейс:

```rust
pub trait PhysicsDomain {
    type Command;
    type Snapshot;
    type Metrics;

    fn prepare(&mut self, ctx: &PrepareContext);
    fn predict(&mut self, ctx: &StepContext);
    fn solve(&mut self, ctx: &StepContext);
    fn commit(&mut self, ctx: &CommitContext);

    fn snapshot(&self) -> Self::Snapshot;
    fn metrics(&self) -> Self::Metrics;
}
```

Реальный GPU hot path не обязан использовать trait objects.

Это архитектурный контракт.

---

# 65. Coupler interface

```rust
pub trait PhysicsCoupler {
    fn detect(&mut self, ctx: &CouplingContext);
    fn exchange(&mut self, ctx: &CouplingContext);
    fn converged(&self) -> bool;
}
```

Strong couplers могут делать несколько iterations.

---

# 66. State ownership registry

Полезно формализовать таблицу:

```rust
pub struct StateOwnership {
    pub quantity: PhysicalQuantity,
    pub owner_domain: DomainId,
    pub readers: Vec<DomainId>,
    pub writers: Vec<DomainId>,
}
```

В production это может быть статическая конфигурация/compile-time policy.

---

# 67. Command flow

Gameplay не должен напрямую менять solver buffers.

```text
Gameplay Command
      ↓
PhysicalWorld command queue
      ↓
domain validation
      ↓
safe application stage
```

Примеры:

```text
ApplyForce
StartCut
IgniteRegion
InjectWater
RemoveMaterial
SetMotorTarget
```

---

# 68. Anti-patterns

## 68.1. `UniversalParticle`

Не создавать particle, который одновременно является:

- водой;
- почвой;
- воздухом;
- деревом;
- огнём;
- rigid body.

Это приведёт к:

- огромному sparse state;
- ветвлениям;
- плохому GPU layout;
- неопределённому ownership.

---

## 68.2. Один глобальный monolithic solver

Не пытаться решить весь мир одной матрицей.

---

## 68.3. Прямое изменение внутренних buffers

Домены взаимодействуют через couplers.

---

## 68.4. Distance-only LOD

Physics relevance шире расстояния.

---

## 68.5. Сохранение всех высокодетальных DOF

Использовать компактное persistent state.

---

## 68.6. Gameplay флаги как основа физики

```text
is_muddy
is_burning
is_broken
```

могут быть semantic summaries, но не заменяют физическое состояние.

---

## 68.7. Research solver без baseline

Каждому эксперименту нужен контрольный backend.

---

# 69. Рекомендуемый порядок реализации

Не начинать с полной абстракции всех будущих доменов.

Строить архитектуру из работающих vertical slices.

---

# 70. Этап 1 — `PhysicalWorldCore`

Создать:

- IDs;
- time;
- domain lifecycle;
- command queue;
- event queue;
- state ownership rules;
- debug metrics.

Без сложной физики.

---

# 71. Этап 2 — Rigid + Continuum

Первый сильный vertical slice:

```text
RigidDomain
↔
ContinuumDomain
```

Сценарии:

- объект падает в воду;
- лодка плавает;
- колесо едет по particle soil;
- предмет вытесняет грязь.

---

# 72. Этап 3 — Vegetation + Rigid

```text
RigidDomain
↔
VegetationDomain
```

Сценарии:

- автомобиль ударяет дерево;
- дерево гнётся;
- дерево ломается;
- detached branch становится rigid.

---

# 73. Этап 4 — Vegetation + ContinuumSoil

```text
VegetationDomain
↔
ContinuumDomain
```

Сценарии:

- корни удерживаются в сухой почве;
- saturated soil ослабевает;
- дерево вырывается;
- root plate деформирует землю.

---

# 74. Этап 5 — Atmosphere

Добавить:

- wind;
- rain;
- humidity.

Сценарии:

```text
wind → tree loads
rain → soil saturation
rain → wood moisture
```

---

# 75. Этап 6 — Thermal / Fire

Добавить:

```text
heat transfer
ignition
wood weakening
evaporation
```

Сценарии:

- мокрое дерево горит хуже;
- горящая ветка ломается;
- вода тушит огонь;
- ветер переносит embers.

---

# 76. Этап 7 — Unified Representation Manager

После появления нескольких доменов формализовать:

- promotion;
- demotion;
- persistent reconstruction;
- resource budgets;
- cross-domain predictive activation.

---

# 77. Первый showcase

Минимальная системная демонстрация:

```text
внедорожник
→ едет по мокрой грязи
→ создаёт колеи
→ теряет сцепление
→ сталкивается с деревом
→ дерево гнётся
→ корни взаимодействуют с слабой почвой
→ дерево вырывается
→ падает в грязь
→ вытесняет воду и почву
```

Это проверяет:

- rigid;
- continuum;
- vegetation;
- coupling;
- topology;
- representation;
- persistence.

---

# 78. Второй showcase

```text
дождь
→ дерево и почва намокают
→ игрок рубит дерево
→ ветер влияет на направление падения
→ дерево падает
→ игрок поджигает сухие ветки
→ огонь распространяется
→ вода тушит часть пожара
```

---

# 79. Production path

```text
Rigid:
mature solver

Continuum:
GPU DFSPH + production soil model

Vegetation:
modal + coarse rods + section fracture

Atmosphere:
procedural fields

Thermal:
reduced branch/material thermal model

Representation:
deterministic state ladders
```

---

# 80. Research path

```text
adaptive variational continuum
DDFK hybrid
two-phase soil-water
deformable tire
strand wood fracture
local volumetric combustion
root-particle soil coupling
learned preconditioners/operators
```

---

# 81. Рекомендуемые ADR

## Core

- `ADR-PhysicalWorld-Architecture.md`
- `ADR-Physics-Domain-Boundaries.md`
- `ADR-Physical-State-Ownership.md`
- `ADR-Physical-Units-And-Time.md`

## Coupling

- `ADR-Coupling-Graph.md`
- `ADR-Coupling-Quantities.md`
- `ADR-Strong-Coupling-Iterations.md`
- `ADR-Topology-Change-Protocol.md`

## Representation

- `ADR-Representation-Promotion.md`
- `ADR-Physics-Relevance-Scoring.md`
- `ADR-State-Transfer.md`
- `ADR-Sleeping-And-Reconstruction.md`

## Materials

- `ADR-Material-Capabilities.md`
- `ADR-Material-State-Ownership.md`

## Runtime

- `ADR-Multi-Rate-Physics-Scheduler.md`
- `ADR-Physics-GPU-Scheduling.md`
- `ADR-PhysicalWorld-Spatial-Index.md`

## Persistence/networking

- `ADR-PhysicalWorld-Persistence.md`
- `ADR-Topology-Event-Replication.md`
- `ADR-Physics-Network-Authority.md`

## Queries

- `ADR-PhysicalWorld-Semantic-Queries.md`
- `ADR-Physics-Events.md`

---

# 82. Рекомендуемые спецификации

## Core

- `SPEC-PhysicalWorld-Core.md`
- `SPEC-Physics-Entity-IDs.md`
- `SPEC-Physics-Command-Queue.md`
- `SPEC-Topology-Transaction-Stage.md`

## Scheduler

- `SPEC-Multi-Rate-Scheduler.md`
- `SPEC-Physics-Dependency-Graph.md`
- `SPEC-Physics-Substeps.md`

## Coupling

- `SPEC-Rigid-Continuum-Coupler.md`
- `SPEC-Rigid-Vegetation-Coupler.md`
- `SPEC-Articulated-Continuum-Coupler.md`
- `SPEC-Vegetation-Soil-Coupler.md`
- `SPEC-Vegetation-Wind-Coupler.md`
- `SPEC-Thermal-Vegetation-Coupler.md`
- `SPEC-Atmosphere-Continuum-Coupler.md`

## Representation

- `SPEC-Representation-Manager.md`
- `SPEC-Relevance-Scoring.md`
- `SPEC-Predictive-Activation.md`
- `SPEC-State-Promotion.md`
- `SPEC-State-Demotion.md`

## Materials

- `SPEC-Material-Registry.md`
- `SPEC-Material-Capability-Schemas.md`
- `SPEC-Material-State.md`

## Persistence

- `SPEC-Physical-Region-Persistence.md`
- `SPEC-Continuum-State-Compression.md`
- `SPEC-Vegetation-State-Persistence.md`
- `SPEC-Topology-Delta-Format.md`

## Networking

- `SPEC-Physics-Network-Events.md`
- `SPEC-Physics-Correction-Snapshots.md`
- `SPEC-Client-Secondary-Physics.md`

## Queries

- `SPEC-Traversability-Queries.md`
- `SPEC-Structural-Hazard-Queries.md`
- `SPEC-Material-Sampling.md`

## Validation

- `SPEC-Cross-Domain-Validation.md`
- `SPEC-Conservation-Metrics.md`
- `SPEC-Physics-Performance-Budgets.md`

---

# 83. Research spikes

- `SPIKE-Rigid-Continuum-Strong-Coupling.md`
- `SPIKE-Deformable-Tire-Mud-Coupling.md`
- `SPIKE-Tree-Root-Particle-Soil.md`
- `SPIKE-Tree-Fall-Water-Impact.md`
- `SPIKE-Representation-State-Transfer.md`
- `SPIKE-Continuum-Sleep-Reconstruction.md`
- `SPIKE-Physics-Network-Replay.md`
- `SPIKE-Multirate-GPU-Scheduling.md`
- `SPIKE-Cross-Domain-Energy-Accounting.md`

---

# 84. Вопросы, которые агент должен решить до детализации

## Architecture

1. Какие домены входят в первую версию?
2. Где проходит граница между domain и coupler?
3. Кто владеет temperature state?
4. Кто применяет topology changes?
5. Как идентифицируются объекты после split/merge?

## Scheduling

6. Как синхронизировать разные update rates?
7. Какие couplers требуют iterations?
8. Где допустим frame latency?
9. Как избежать CPU readback?

## Representation

10. Какой общий interface нужен state transfer?
11. Как representation budgets распределяются по GPU?
12. Как predictive activation взаимодействует со streaming?

## Persistence

13. Как сжимать active continuum state?
14. Как хранить long-term plastic soil history?
15. Как сохранять partially cut tree?
16. Как сохранять fallen debris?

## Networking

17. Какие topology events authoritative?
18. Какие continuous dynamics могут быть client-local?
19. Нужна ли rollback compatibility?

## AI

20. Какие физические queries нужны Strategic/Tactical/Motor уровням?
21. Как обновлять traversability без чтения particle state?
22. Как AI узнаёт об опасности падающего дерева заранее?

---

# 85. Критерии успеха архитектуры

Архитектура успешна, если:

1. Новый domain можно добавить без прямого доступа ко всем остальным.
2. Каждый физический quantity имеет одного владельца.
3. Couplers можно тестировать отдельно.
4. Representation можно менять без видимого скачка.
5. Спящие регионы не требуют высокодетальных DOF.
6. Topology events сохраняются и реплицируются.
7. AI получает семантическое состояние.
8. Conservation errors измеряются.
9. Research solver можно заменить baseline solver.
10. Cross-domain demo работает без bespoke сценарного кода.

---

# 86. Нормативная формулировка для будущих спецификаций

Будущие спецификации должны следовать правилам:

- `MUST` — обязательное архитектурное требование;
- `SHOULD` — предпочтительное решение;
- `MAY` — допустимое расширение.

Базовые требования:

1. `PhysicalWorld` **MUST NOT** быть monolithic solver.
2. Каждый domain **MUST** иметь явного владельца состояния.
3. Междоменные воздействия **MUST** проходить через couplers.
4. Topology changes **MUST** применяться в безопасной transactional stage.
5. Representation transitions **MUST** сохранять gameplay-relevant state.
6. Спящие объекты **MUST NOT** требовать хранения всех active DOF.
7. Gameplay **MUST NOT** напрямую изменять internal solver buffers.
8. AI **SHOULD** использовать semantic queries.
9. Каждый research backend **SHOULD** иметь baseline comparison.
10. Cross-domain conservation **MUST** измеряться.

---

# 87. Финальная архитектурная схема

```text
                              GAMEPLAY / AI
                                    │
                     Commands / Queries / Events
                                    │
                                    ▼
                         ┌────────────────────┐
                         │   PHYSICAL WORLD   │
                         └────────────────────┘
                                    │
      ┌─────────────────────────────┼─────────────────────────────┐
      │                             │                             │
 Simulation Scheduler        Coupling Graph          Representation Manager
      │                             │                             │
      │                  impulse / heat / mass                   │
      │                  moisture / topology                    │
      ▼                             ▼                             ▼
┌───────────┐  ┌────────────┐  ┌────────────┐  ┌─────────────┐
│   Rigid   │  │Articulated │  │ Continuum  │  │ Vegetation  │
└───────────┘  └────────────┘  └────────────┘  └─────────────┘
      │              │                │                 │
      └──────────────┼────────────────┼─────────────────┘
                     │                │
                     ▼                ▼
              ┌────────────┐   ┌────────────┐
              │ Atmosphere │   │  Thermal   │
              └────────────┘   └────────────┘
                     │                │
                     └───────┬────────┘
                             ▼
                    Material / Fracture
                             │
                 Persistence / Replication
```

---

# 88. Итог

Все исследованные подсистемы действительно должны войти в единую физическую систему мира.

Но объединять необходимо:

```text
ownership
coupling
materials
time
representation
streaming
persistence
queries
validation
```

а не сами solvers в одну математическую модель.

Правильный итог:

> **PhysicalWorld создаёт единый причинно связанный физический мир, в котором вода, почва, грязь, деревья, ветер, огонь, rigid bodies, articulated characters и транспорт остаются специализированными доменами, но взаимодействуют через общие физические контракты и согласованное состояние.**

Первый практический vertical slice:

```text
Rigid
↔ Continuum
↔ Vegetation
```

с минимальным `Atmosphere`:

```text
внедорожник
→ мокрая почва
→ колеи
→ столкновение с деревом
→ ослабленные корни
→ падение дерева
→ вытеснение грязи и воды
```

Именно этот сценарий должен стать первой проверкой того, что `PhysicalWorld` является не набором отдельных эффектов, а настоящим системным физическим слоем движка.

---

# 89. Одно предложение для project thesis

> **Создать для Rust-движка единый PhysicalWorld layer, который оркестрирует специализированные rigid, articulated, continuum, vegetation, atmosphere и thermal solvers через консервативные couplers, representation promotion, persistent material state и semantic queries, позволяя сложному поведению мира возникать из взаимодействия физических систем, а не из заранее прописанных реакций.**
