namespace {

constexpr const char* B4E2D7R19R50_IDENTITY_SHA256 =
    "ab745dfcfede239ed739d5bec80f1bc6c9e74be36b3bd5daa6887f830f263c75";
constexpr const char* B4E2D7R19R50_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r50-active-face-hildreth-closure|v1|parent=8789cc16:7e1dde137b6164f67a448ab704eb713148e08c3a3a6e693614cb2645b01fa656:bbdfae6fbd301b74dbdbfde8bc416f814cf7e06ec74b8b54c8abd7718dac6544:RESTORATION_HZ_PRIMAL_ACCELERATOR_CANDIDATE|source=r49-witness-7716e30c95b802a36f8c7d62dc993c5d912e678ec4fcb1617e364fba7898ded5;checkpoint-b19e6e64eaf68e9bd8eccf748d89cda661b42850118d355846b92f329975caba;steps-ec1fcee08dd724d98b0ce4e5506fdc75e90d654e122f65dcb2afcb8436aaade7;active366;maximum-upper1.9771445876661678e-11|problem=directed-active-majorant-minimum-norm-halfspace-projection;unchanged-r48-A-contact-box-normal-ball;working-rows=current-directed-upper-positive;u=current-directed-upper;B=restrict-working-A;min0.5*norm2p;u+Bp<=0|capacity=unique-row-cache512;next-power-of-two-above-parent366;overflow-reject;never-truncate|basis=row-gradient=At-ei-one-vjp;gram-column=A-row-gradient-one-jvp;cache-fixed-operator-across-outers;positive-diagonal;relative-symmetry<=1e-12|solver=hildreth-dual-coordinate;lambda0=0-each-outer;correction0;stable-ascending-row-order;coordinate=lambda-new=max(0,lambda+predicted/diagonal);exact-eight-sweeps;checkpoints1,2,4,8;no-tolerance-stop|globalization=target=current+correction;project-r48-ball-box;feasible-chord;dyadic-alpha1-through2^-8-largest-first;fresh-directed-audit-each;accept-first-primal-certified-or-strict-psi-h-maximum-upper-decrease|max-outers4;rebuild-positive-upper-active-set-after-accept;no-fifth-outer|certificate=unchanged-r48-fresh-all-row-directed-jvp;zero-certified-positive-only;parent-dual-unchanged;no-infeasibility-claim|selection=certificate-first;else-active-face-closure-candidate-if-at-least-one-accepted-outer-and-terminal-strict-psi-h-maximum-upper-dominance;else-reference-retained|controls=r49-parent-bytes;dense-hildreth-projection;dense-dual-release;dense-active-addition;dense-capacity;dense-symmetric-gram;dense-dyadic-globalization;directed-rounding;source;workspace;geometry;active-cache;gram;hildreth;projection;globalization;certificate;work;rollback;route-precedence|routes=active-face-parent-rejected;active-face-source-rejected;active-face-workspace-rejected;active-face-geometry-rejected;active-face-capacity-rejected;active-face-gram-rejected;active-face-hildreth-rejected;active-face-projection-rejected;active-face-globalization-rejected;active-face-primal-audit-rejected;active-face-work-rejected;restoration-next-trqp-compatible-candidate;active-face-hildreth-closure-candidate;active-face-reference-retained|precedence=parent,source,workspace,geometry,capacity,gram,hildreth,projection,globalization,primal-audit,work,certificate,selection|work=parent-r49-replays1;static-index1;superset-build1;moved-workspace1;releases1;unique-rows<=512;row-vjp<=512;gram-jvp<=512;initial-directed-jvp1;candidate-directed-jvp<=36;pairpasses<=1061;outers<=4;sweeps<=32;projection-calls<=4;new-support-audits0;new-hvp0;new-model0;new-nonlinear-trial0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46-r47-r48-r49=unchanged;normal-witness-apply=none;r43-restoration-commit=none;filter-runtime-commit=none;restoration-exit=none;switching=none;trust-update=none;following-outer=none;tolerance=none;penalty-change=none;support-expansion=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-active-face-closure-classification-only)IDENTITY";

constexpr std::size_t AL_R50_ROW_CAPACITY = 512U;
constexpr std::size_t AL_R50_SWEEPS = 8U;
constexpr std::size_t AL_R50_OUTERS = 4U;
constexpr std::size_t AL_R50_DYADIC_TRIALS = 9U;
constexpr double AL_R50_GRAM_SYMMETRY = 1.0e-12;

struct ALR50Audit {
    bool passed = false;
    ALR48PrimalAudit primal;
    ALLinearizedJvp image;
    std::vector<double> upper;
    std::vector<std::size_t> active;
    std::string active_root;
};

ALR50Audit al_r50_audit(const ALNormalizedWorkspace& workspace,
    const std::vector<Vec3>& witness) {
    ALR50Audit result;
    result.image = al_r29_directed_jvp(workspace, witness);
    result.primal = al_r48_primal_audit(workspace.support.constraint,
        result.image, workspace.neighborhood.maximum_degree);
    if (!result.image.passed || !result.primal.passed) return result;
    const std::vector<double>& c = workspace.support.constraint;
    const double gamma = gamma_factor(
        16U * workspace.neighborhood.maximum_degree + 66U);
    result.upper.resize(c.size());
    std::vector<std::uint8_t> mask(c.size(), 0U);
    for (std::size_t row = 0U; row < c.size(); ++row) {
        const double raw = c[row] + result.image.value[row];
        const double bound = gamma
            * (std::abs(c[row]) + result.image.absolute_sum[row]);
        result.upper[row] = raw + bound;
        if (result.upper[row] > 0.0) {
            result.active.push_back(row);
            mask[row] = 1U;
        }
    }
    result.active_root = al_r29_mask_root(mask);
    result.passed = al_r30_finite(result.upper)
        && result.active.size() == result.primal.certified_positive;
    return result;
}

struct ALR50RowBasis {
    std::size_t row = 0U;
    std::vector<Vec3> gradient;
    std::vector<double> gram_column;
    double diagonal = 0.0;
    std::string root;
};

std::size_t al_r50_find_basis(
    const std::vector<ALR50RowBasis>& cache, std::size_t row) {
    for (std::size_t index = 0U; index < cache.size(); ++index)
        if (cache[index].row == row) return index;
    return cache.size();
}

bool al_r50_cache_rows(const ALNormalizedWorkspace& workspace,
    const std::vector<std::size_t>& rows,
    std::vector<ALR50RowBasis>* cache,
    std::size_t* vjp_calls, std::size_t* jvp_calls) {
    if (cache == nullptr || vjp_calls == nullptr || jvp_calls == nullptr)
        return false;
    std::size_t required = cache->size();
    for (std::size_t row : rows)
        if (al_r50_find_basis(*cache, row) == cache->size()) ++required;
    if (required > AL_R50_ROW_CAPACITY) return false;
    for (std::size_t row : rows) {
        if (al_r50_find_basis(*cache, row) != cache->size()) continue;
        std::vector<double> unit(workspace.support.constraint.size(), 0.0);
        unit[row] = 1.0;
        ALR50RowBasis basis;
        basis.row = row;
        basis.gradient = al_r29_pair_once_vjp(workspace, unit);
        ++*vjp_calls;
        if (basis.gradient.size() != workspace.support.constraint.size()
            || !al_r48_vec_finite(basis.gradient)) return false;
        const ALLinearizedJvp gram =
            al_r29_pair_once_jvp(workspace, basis.gradient);
        ++*jvp_calls;
        if (!gram.passed
            || gram.value.size() != workspace.support.constraint.size())
            return false;
        basis.gram_column = gram.value;
        basis.diagonal = basis.gram_column[row];
        const long double direct_diagonal =
            al_r35_dot_vec3(basis.gradient, basis.gradient);
        const double diagonal_scale = std::max({
            std::abs(basis.diagonal),
            std::abs(static_cast<double>(direct_diagonal)), 1.0e-300});
        const bool diagonal_exact = std::isfinite(basis.diagonal)
            && basis.diagonal > 0.0
            && std::abs(basis.diagonal
                - static_cast<double>(direct_diagonal))
                <= AL_R50_GRAM_SYMMETRY * diagonal_scale;
        if (!diagonal_exact) return false;
        std::ostringstream projection;
        projection << row << ':' << binary64_bits(basis.diagonal) << ':'
                   << al_binary64_vec3_root(basis.gradient) << ':'
                   << al_r10_dual_root(basis.gram_column);
        basis.root = sha256_hex(projection.str());
        cache->push_back(std::move(basis));
    }
    return true;
}

struct ALR50Core {
    bool passed = false;
    std::vector<double> lambda;
    std::vector<double> predicted;
    std::size_t sweeps = 0U;
    double predicted_maximum = std::numeric_limits<double>::infinity();
    std::vector<std::string> checkpoint_roots;
};

ALR50Core al_r50_hildreth_core(const std::vector<double>& gram,
    const std::vector<double>& upper, std::size_t sweeps = AL_R50_SWEEPS) {
    ALR50Core result;
    const std::size_t count = upper.size();
    if (count == 0U || gram.size() != count * count
        || !al_r30_finite(gram) || !al_r30_finite(upper)
        || sweeps == 0U) return result;
    result.lambda.assign(count, 0.0);
    result.predicted = upper;
    bool exact = true;
    for (std::size_t sweep = 1U; sweep <= sweeps && exact; ++sweep) {
        for (std::size_t i = 0U; i < count; ++i) {
            const double diagonal = gram[i * count + i];
            if (!std::isfinite(diagonal) || diagonal <= 0.0) {
                exact = false;
                break;
            }
            const double next = std::max(
                0.0, result.lambda[i] + result.predicted[i] / diagonal);
            const double delta = next - result.lambda[i];
            if (!std::isfinite(next) || !std::isfinite(delta)) {
                exact = false;
                break;
            }
            result.lambda[i] = next;
            if (delta != 0.0)
                for (std::size_t row = 0U; row < count; ++row)
                    result.predicted[row] -=
                        delta * gram[row * count + i];
        }
        if (!al_r30_finite(result.lambda)
            || !al_r30_finite(result.predicted)) exact = false;
        if (sweep == 1U || sweep == 2U || sweep == 4U || sweep == 8U) {
            std::ostringstream projection;
            projection << sweep << ':' << al_r10_dual_root(result.lambda)
                       << ':' << al_r10_dual_root(result.predicted);
            result.checkpoint_roots.push_back(
                sha256_hex(projection.str()));
        }
        result.sweeps = sweep;
    }
    result.predicted_maximum = *std::max_element(
        result.predicted.begin(), result.predicted.end());
    result.passed = exact && result.sweeps == sweeps
        && result.checkpoint_roots.size() == 4U
        && std::all_of(result.lambda.begin(), result.lambda.end(),
            [](double value) { return std::isfinite(value) && value >= 0.0; })
        && std::isfinite(result.predicted_maximum);
    return result;
}

struct ALR50Solve {
    bool passed = false;
    bool gram_exact = false;
    ALR50Core core;
    std::vector<Vec3> correction;
    double minimum_diagonal = std::numeric_limits<double>::infinity();
    double maximum_symmetry_relative = 0.0;
    std::string gram_root;
    std::string root;
};

ALR50Solve al_r50_solve(const std::vector<ALR50RowBasis>& cache,
    const ALR50Audit& audit) {
    ALR50Solve result;
    const std::size_t count = audit.active.size();
    if (!audit.passed || count == 0U || cache.empty()) return result;
    std::vector<std::size_t> basis_index(count);
    for (std::size_t i = 0U; i < count; ++i) {
        basis_index[i] = al_r50_find_basis(cache, audit.active[i]);
        if (basis_index[i] == cache.size()) return result;
    }
    std::vector<double> gram(count * count);
    std::vector<double> upper(count);
    bool gram_exact = true;
    for (std::size_t i = 0U; i < count; ++i) {
        upper[i] = audit.upper[audit.active[i]];
        for (std::size_t j = 0U; j < count; ++j) {
            gram[j * count + i] =
                cache[basis_index[i]].gram_column[audit.active[j]];
        }
        result.minimum_diagonal = std::min(
            result.minimum_diagonal, gram[i * count + i]);
    }
    for (std::size_t i = 0U; i < count; ++i)
        for (std::size_t j = i + 1U; j < count; ++j) {
            const double lhs = gram[i * count + j];
            const double rhs = gram[j * count + i];
            const double scale = std::max({
                std::abs(lhs), std::abs(rhs), 1.0e-300});
            const double relative = std::abs(lhs - rhs) / scale;
            result.maximum_symmetry_relative = std::max(
                result.maximum_symmetry_relative, relative);
            gram_exact = gram_exact && std::isfinite(relative)
                && relative <= AL_R50_GRAM_SYMMETRY;
        }
    gram_exact = gram_exact && std::isfinite(result.minimum_diagonal)
        && result.minimum_diagonal > 0.0;
    result.gram_exact = gram_exact;
    if (!gram_exact) return result;
    result.core = al_r50_hildreth_core(gram, upper);
    if (!result.core.passed) return result;
    result.correction.assign(cache.front().gradient.size(), Vec3{});
    for (std::size_t i = 0U; i < count; ++i) {
        const std::vector<Vec3>& gradient = cache[basis_index[i]].gradient;
        for (std::size_t particle = 0U;
             particle < result.correction.size(); ++particle) {
            result.correction[particle] = result.correction[particle]
                - result.core.lambda[i] * gradient[particle];
        }
    }
    std::ostringstream gram_projection;
    gram_projection << count << ':' << al_r10_dual_root(gram) << ':'
                    << binary64_bits(result.minimum_diagonal) << ':'
                    << binary64_bits(result.maximum_symmetry_relative);
    result.gram_root = sha256_hex(gram_projection.str());
    std::ostringstream projection;
    projection << result.gram_root << ':'
               << result.core.checkpoint_roots.size() << ':';
    for (const std::string& root : result.core.checkpoint_roots)
        projection << root << ':';
    projection << al_binary64_vec3_root(result.correction) << ':'
               << binary64_bits(result.core.predicted_maximum);
    result.root = sha256_hex(projection.str());
    result.passed = al_r48_vec_finite(result.correction);
    return result;
}

struct ALR50OuterRecord {
    std::size_t outer = 0U;
    std::size_t active = 0U;
    std::size_t cache_rows = 0U;
    std::size_t selected_dyadic = 0U;
    double alpha = 0.0;
    double predicted_maximum = 0.0;
    double maximum_symmetry_relative = 0.0;
    double psi = 0.0;
    double h = 0.0;
    double maximum_upper = 0.0;
    std::string active_root;
    std::string solve_root;
    std::string root;
};

struct ALR50Generator {
    bool passed = false;
    bool certified = false;
    bool capacity_exact = true;
    bool gram_exact = true;
    bool hildreth_exact = true;
    bool projection_exact = true;
    bool globalization_exact = true;
    bool primal_exact = true;
    std::size_t accepted_outers = 0U;
    std::size_t total_sweeps = 0U;
    std::size_t row_vjp = 0U;
    std::size_t gram_jvp = 0U;
    std::size_t audit_jvp = 0U;
    std::size_t projection_calls = 0U;
    std::size_t projection_scans = 0U;
    std::size_t pair_passes = 0U;
    double witness_norm = 0.0;
    ALR50Audit source;
    ALR50Audit terminal;
    std::vector<Vec3> witness;
    std::vector<ALR50RowBasis> cache;
    std::vector<ALR50OuterRecord> records;
};

ALR50Generator al_r50_generate(const ALNormalizedWorkspace& workspace,
    const std::vector<Vec3>& source_witness,
    const std::vector<Vec3>& lower, const std::vector<Vec3>& upper) {
    ALR50Generator result;
    if (!workspace.passed || !al_r49_feasible(source_witness, lower, upper))
        return result;
    std::vector<Vec3> current = source_witness;
    result.source = al_r50_audit(workspace, current);
    ++result.audit_jvp;
    result.terminal = result.source;
    if (!result.source.passed) return result;
    for (std::size_t outer = 1U;
         outer <= AL_R50_OUTERS && !result.certified; ++outer) {
        if (result.terminal.active.empty()) {
            result.certified = true;
            break;
        }
        const std::size_t old_cache = result.cache.size();
        const bool cached = al_r50_cache_rows(workspace,
            result.terminal.active, &result.cache,
            &result.row_vjp, &result.gram_jvp);
        result.capacity_exact = result.capacity_exact && cached
            && result.cache.size() <= AL_R50_ROW_CAPACITY;
        if (!cached) break;
        const ALR50Solve solve = al_r50_solve(result.cache, result.terminal);
        result.gram_exact = result.gram_exact && solve.gram_exact;
        result.hildreth_exact = result.hildreth_exact && solve.passed
            && solve.core.sweeps == AL_R50_SWEEPS;
        if (!solve.passed) break;
        result.total_sweeps += solve.core.sweeps;
        std::vector<Vec3> target(current.size());
        for (std::size_t i = 0U; i < current.size(); ++i)
            target[i] = current[i] + solve.correction[i];
        const ALR48Projection projection = al_r48_project_ball_box(
            target, lower, upper, AL_R48_NORMAL_RADIUS);
        ++result.projection_calls;
        result.projection_scans += projection.scans;
        result.projection_exact = result.projection_exact && projection.passed;
        if (!projection.passed) break;
        std::vector<Vec3> chord(current.size());
        for (std::size_t i = 0U; i < current.size(); ++i)
            chord[i] = projection.value[i] - current[i];
        const double chord_norm = al_r32_vec3_norm(chord);
        if (!std::isfinite(chord_norm) || chord_norm == 0.0) {
            result.globalization_exact = false;
            break;
        }
        bool selected = false;
        std::size_t selected_dyadic = 0U;
        double selected_alpha = 0.0;
        ALR50Audit selected_audit;
        std::vector<Vec3> selected_witness;
        for (std::size_t dyadic = 0U;
             dyadic < AL_R50_DYADIC_TRIALS; ++dyadic) {
            const double alpha = std::ldexp(1.0, -static_cast<int>(dyadic));
            std::vector<Vec3> candidate(current.size());
            for (std::size_t i = 0U; i < current.size(); ++i)
                candidate[i] = current[i] + alpha * chord[i];
            if (!al_r49_feasible(candidate, lower, upper)) {
                result.projection_exact = false;
                break;
            }
            ALR50Audit audit = al_r50_audit(workspace, candidate);
            ++result.audit_jvp;
            const bool certified = audit.passed && audit.active.empty();
            const bool strict = audit.passed
                && audit.primal.psi < result.terminal.primal.psi
                && audit.primal.h < result.terminal.primal.h
                && audit.primal.maximum_upper
                    < result.terminal.primal.maximum_upper;
            if (certified || strict) {
                selected = true;
                selected_dyadic = dyadic;
                selected_alpha = alpha;
                selected_audit = std::move(audit);
                selected_witness = std::move(candidate);
                break;
            }
        }
        result.globalization_exact = result.globalization_exact && selected;
        if (!result.projection_exact || !selected) break;
        current = std::move(selected_witness);
        result.terminal = std::move(selected_audit);
        ++result.accepted_outers;
        result.certified = result.terminal.active.empty();
        result.primal_exact = result.primal_exact && result.terminal.passed;
        ALR50OuterRecord record;
        record.outer = outer;
        record.active = result.terminal.active.size();
        record.cache_rows = result.cache.size();
        record.selected_dyadic = selected_dyadic;
        record.alpha = selected_alpha;
        record.predicted_maximum = solve.core.predicted_maximum;
        record.maximum_symmetry_relative = solve.maximum_symmetry_relative;
        record.psi = result.terminal.primal.psi;
        record.h = result.terminal.primal.h;
        record.maximum_upper = result.terminal.primal.maximum_upper;
        record.active_root = result.terminal.active_root;
        record.solve_root = solve.root;
        std::ostringstream record_projection;
        record_projection << outer << ':' << old_cache << ':'
                          << record.cache_rows << ':' << selected_dyadic << ':'
                          << binary64_bits(selected_alpha) << ':'
                          << binary64_bits(record.predicted_maximum) << ':'
                          << binary64_bits(record.maximum_symmetry_relative)
                          << ':' << binary64_bits(record.psi) << ':'
                          << binary64_bits(record.h) << ':'
                          << binary64_bits(record.maximum_upper) << ':'
                          << record.active << ':' << record.active_root << ':'
                          << record.solve_root;
        record.root = sha256_hex(record_projection.str());
        result.records.push_back(std::move(record));
    }
    result.witness = std::move(current);
    result.witness_norm = al_r32_vec3_norm(result.witness);
    result.pair_passes = result.row_vjp + result.gram_jvp + result.audit_jvp;
    const bool terminal_state = result.certified
        || result.accepted_outers == AL_R50_OUTERS;
    result.passed = result.source.passed && result.source.active.size() == 366U
        && result.capacity_exact && result.gram_exact && result.hildreth_exact
        && result.projection_exact && result.globalization_exact
        && result.primal_exact && result.terminal.passed && terminal_state
        && result.cache.size() <= AL_R50_ROW_CAPACITY
        && result.row_vjp <= 512U && result.gram_jvp <= 512U
        && result.audit_jvp <= 37U && result.pair_passes <= 1061U
        && result.accepted_outers <= AL_R50_OUTERS
        && result.total_sweeps <= AL_R50_OUTERS * AL_R50_SWEEPS
        && result.projection_calls <= AL_R50_OUTERS
        && al_r49_feasible(result.witness, lower, upper)
        && std::isfinite(result.witness_norm);
    return result;
}

bool al_r50_dense_controls(std::string* output_root) {
    const ALR50Core identity = al_r50_hildreth_core(
        {1.0, 0.0, 0.0, 1.0}, {1.0, 2.0});
    const ALR50Core release = al_r50_hildreth_core(
        {1.0, 0.0, 0.0, 1.0}, {-1.0, 1.0});
    const ALR50Core coupled = al_r50_hildreth_core(
        {2.0, 1.0, 1.0, 2.0}, {1.0, 1.0});
    const bool capacity = 366U <= AL_R50_ROW_CAPACITY
        && 513U > AL_R50_ROW_CAPACITY;
    const bool dyadic = std::ldexp(1.0, 0) == 1.0
        && std::ldexp(1.0, -8) == 0.00390625;
    const bool passed = identity.passed && release.passed && coupled.passed
        && identity.predicted_maximum == 0.0
        && release.lambda[0] == 0.0 && release.lambda[1] == 1.0
        && release.predicted_maximum == 0.0
        && coupled.predicted_maximum <= 0.0 && capacity && dyadic;
    std::ostringstream projection;
    projection << passed << ':' << al_r10_dual_root(identity.lambda) << ':'
               << al_r10_dual_root(identity.predicted) << ':'
               << al_r10_dual_root(release.lambda) << ':'
               << al_r10_dual_root(coupled.predicted) << ':'
               << identity.checkpoint_roots.size() << ':' << capacity << ':'
               << dyadic;
    if (output_root != nullptr) *output_root = sha256_hex(projection.str());
    return passed;
}

std::string al_r50_route(bool parent, bool source, bool workspace,
    bool geometry, bool capacity, bool gram, bool hildreth,
    bool projection, bool globalization, bool primal, bool work,
    bool certified, bool closure) {
    if (!parent) return "ACTIVE_FACE_PARENT_REJECTED";
    if (!source) return "ACTIVE_FACE_SOURCE_REJECTED";
    if (!workspace) return "ACTIVE_FACE_WORKSPACE_REJECTED";
    if (!geometry) return "ACTIVE_FACE_GEOMETRY_REJECTED";
    if (!capacity) return "ACTIVE_FACE_CAPACITY_REJECTED";
    if (!gram) return "ACTIVE_FACE_GRAM_REJECTED";
    if (!hildreth) return "ACTIVE_FACE_HILDRETH_REJECTED";
    if (!projection) return "ACTIVE_FACE_PROJECTION_REJECTED";
    if (!globalization) return "ACTIVE_FACE_GLOBALIZATION_REJECTED";
    if (!primal) return "ACTIVE_FACE_PRIMAL_AUDIT_REJECTED";
    if (!work) return "ACTIVE_FACE_WORK_REJECTED";
    if (certified) return "RESTORATION_NEXT_TRQP_COMPATIBLE_CANDIDATE";
    if (closure) return "ACTIVE_FACE_HILDRETH_CLOSURE_CANDIDATE";
    return "ACTIVE_FACE_REFERENCE_RETAINED";
}

struct ALR50ReplayCapture {
    bool captured = false;
    ALR49ReplayCapture r49;
    ALR50Generator generator;
    std::string cache_root;
    std::string outer_root;
    std::string witness_root;
};

} // namespace

SplitBoundaryReport run_al_active_face_closure_controls_impl(
    ALR50ReplayCapture* replay_capture) {
    const bool identity = sha256_hex(B4E2D7R19R50_IDENTITY_PROJECTION)
        == B4E2D7R19R50_IDENTITY_SHA256;
    ALR49ReplayCapture capture;
    const SplitBoundaryReport parent =
        run_al_restoration_hz_accelerator_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_exact = parent.passed && capture.captured
        && parent_stdout
            == "7e1dde137b6164f67a448ab704eb713148e08c3a3a6e693614cb2645b01fa656"
        && parent.json.find(
            "bbdfae6fbd301b74dbdbfde8bc416f814cf7e06ec74b8b54c8abd7718dac6544")
            != std::string::npos
        && parent.json.find("RESTORATION_HZ_PRIMAL_ACCELERATOR_CANDIDATE")
            != std::string::npos;
    ALR43ReplayCapture& r43 = capture.r48.r47.r46.r45.r44.r43;
    ALR40ReplayCapture& r40 = r43.r42.r40;
    ALR30RangeProjectionCapture& r30 =
        r40.r39.r38.r37.r36.r35.r34.r33.r32.r31.r30;
    const std::string position_before = al_binary64_vec3_root(r30.position);
    const std::string trial_before =
        al_binary64_vec3_root(r43.projected_trial);
    const std::string owner_before = al_r29_mask_root(r43.owners);
    const std::string superset_before =
        al_r42_superset_root(r43.r42.superset);
    const bool source_exact = parent_exact && r30.captured
        && capture.witness_root
            == "7716e30c95b802a36f8c7d62dc993c5d912e678ec4fcb1617e364fba7898ded5"
        && capture.checkpoint_root
            == "b19e6e64eaf68e9bd8eccf748d89cda661b42850118d355846b92f329975caba"
        && capture.step_root
            == "ec1fcee08dd724d98b0ce4e5506fdc75e90d654e122f65dcb2afcb8436aaade7"
        && capture.primal.certified_positive == 366U
        && binary64_bits(capture.primal.maximum_upper)
            == binary64_bits(1.9771445876661678e-11)
        && position_before
            == "e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828"
        && trial_before
            == "0fb11d7feb63a38f285798bd1eb2aad49131fbe5ff72df48e2230b31b8ffcc50"
        && owner_before
            == "baf145ae5d293802b8a1915ec05b10781b79bfed2f30f520f0a960bb817e5187"
        && superset_before
            == "355ce6fa5ccaf7524e2f28fedf74c68baf0e85a60429a5442d3a4bd707130f01";

    std::string dense_root;
    const bool dense = identity && al_r50_dense_controls(&dense_root);
    ALSparseWorkTrace workspace_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    const SmokeFixture fixture = make_b4e2d_dam_fixture();
    const JointStaticSupportIndex index = build_joint_static_support_index(
        tagged_points(fixture.boundary), &static_work);
    const JointStaticSupportBinding binding = bind_joint_static_support_index(
        &index, index.identity_sha256);
    JointNeighborhood superset = b4ep3_build_superset(
        tagged_points(r30.position), binding);
    JointNeighborhood moved_filtered = b4ep3_filter_superset(
        tagged_points(r43.projected_trial), superset);
    ALNormalizedWorkspace moved = build_al_normalized_sparse_workspace(
        r43.projected_trial, binding, r30.dual, r30.theta,
        &workspace_work, &static_work, &adjacency_work);
    const ALR40HingeMetric moved_metric =
        al_r40_hinge_metric(moved.support.constraint);
    const bool workspace_exact = source_exact && dense && index.passed
        && binding.passed && superset.passed && moved.passed
        && al_r42_superset_root(superset) == superset_before
        && moved_filtered.pairs == moved.neighborhood.pairs
        && moved_metric.finite
        && binary64_bits(moved_metric.psi)
            == binary64_bits(6.8540208964482797e-16);

    std::vector<Vec3> lower(r43.projected_trial.size());
    std::vector<Vec3> upper(r43.projected_trial.size());
    std::vector<std::uint8_t> moved_owners(
        r43.projected_trial.size() * 3U, 0U);
    std::size_t moved_active_faces = 0U;
    bool intervals_exact = workspace_exact;
    for (std::size_t i = 0U; i < r43.projected_trial.size(); ++i)
        for (int axis = 0; axis < 3; ++axis) {
            const double y = al_r41_component(r43.projected_trial[i], axis);
            const double low_face = al_r41_component(fixture.contact_low, axis);
            const double high_face = al_r41_component(
                fixture.contact_high, axis);
            double low = (low_face - y) / SPACING;
            double high = (high_face - y) / SPACING;
            const std::size_t component =
                3U * i + static_cast<std::size_t>(axis);
            if (y < low_face) {
                low = 0.0;
                moved_owners[component] = 1U;
                ++moved_active_faces;
            } else if (y > high_face) {
                high = 0.0;
                moved_owners[component] = 2U;
                ++moved_active_faces;
            }
            al_r43_component(lower[i], axis) = low;
            al_r43_component(upper[i], axis) = high;
            intervals_exact = intervals_exact && std::isfinite(low)
                && std::isfinite(high) && low <= 0.0 && high >= 0.0;
        }
    const double moved_step_norm = al_r32_vec3_norm(r43.projected_endpoint);
    const double half_skin = 0.5 * B4EP3_SKIN
        * std::sqrt(1.0 - 2.0e-12) / SPACING;
    const std::string moved_owner_root = al_r29_mask_root(moved_owners);
    const bool geometry_exact = intervals_exact
        && moved_step_norm + AL_R48_NORMAL_RADIUS <= half_skin
        && moved_active_faces == 1290U && moved_owner_root == owner_before
        && al_r49_feasible(capture.witness, lower, upper);

    ALR50Generator generator;
    if (geometry_exact)
        generator = al_r50_generate(moved, capture.witness, lower, upper);
    const bool generator_exact = geometry_exact && generator.passed;
    const bool capacity_exact = generator_exact && generator.capacity_exact;
    const bool gram_exact = generator_exact && generator.gram_exact;
    const bool hildreth_exact = generator_exact && generator.hildreth_exact;
    const bool projection_exact = generator_exact && generator.projection_exact;
    const bool globalization_exact = generator_exact
        && generator.globalization_exact;
    const bool primal_exact = generator_exact && generator.primal_exact
        && generator.terminal.passed
        && (!generator.certified || generator.terminal.active.empty());
    const bool closure = primal_exact && !generator.certified
        && generator.accepted_outers > 0U
        && generator.terminal.primal.psi < generator.source.primal.psi
        && generator.terminal.primal.h < generator.source.primal.h
        && generator.terminal.primal.maximum_upper
            < generator.source.primal.maximum_upper;
    if (moved.passed) release_al_normalized_workspace(moved, &workspace_work);
    const bool work_exact = workspace_work.workspace_builds == 1U
        && workspace_work.workspace_releases == 1U
        && workspace_work.live_workspaces == 0U
        && workspace_work.maximum_live_workspaces == 1U
        && !workspace_work.lifecycle_underflow
        && workspace_work.all_pair_candidate_calls == 0U
        && static_work.static_index_builds == 1U
        && static_work.support_canonicalizations == 1U
        && static_work.workspace_builds == 1U
        && adjacency_work.workspace_builds == 1U
        && generator.cache.size() <= 512U
        && generator.row_vjp <= 512U && generator.gram_jvp <= 512U
        && generator.audit_jvp <= 37U
        && generator.pair_passes <= 1061U
        && generator.accepted_outers <= 4U
        && generator.total_sweeps <= 32U
        && generator.projection_calls <= 4U;
    const bool rollback = position_before == al_binary64_vec3_root(r30.position)
        && trial_before == al_binary64_vec3_root(r43.projected_trial)
        && owner_before == al_r29_mask_root(r43.owners)
        && superset_before == al_r42_superset_root(r43.r42.superset)
        && capture.witness_root == al_binary64_vec3_root(capture.witness);

    const std::array<const char*, 14U> expected_routes{{
        "ACTIVE_FACE_PARENT_REJECTED", "ACTIVE_FACE_SOURCE_REJECTED",
        "ACTIVE_FACE_WORKSPACE_REJECTED", "ACTIVE_FACE_GEOMETRY_REJECTED",
        "ACTIVE_FACE_CAPACITY_REJECTED", "ACTIVE_FACE_GRAM_REJECTED",
        "ACTIVE_FACE_HILDRETH_REJECTED", "ACTIVE_FACE_PROJECTION_REJECTED",
        "ACTIVE_FACE_GLOBALIZATION_REJECTED",
        "ACTIVE_FACE_PRIMAL_AUDIT_REJECTED", "ACTIVE_FACE_WORK_REJECTED",
        "RESTORATION_NEXT_TRQP_COMPATIBLE_CANDIDATE",
        "ACTIVE_FACE_HILDRETH_CLOSURE_CANDIDATE",
        "ACTIVE_FACE_REFERENCE_RETAINED"}};
    bool routes_exact = true;
    std::ostringstream route_material;
    for (std::size_t route_index = 0U;
         route_index < expected_routes.size(); ++route_index) {
        std::array<bool, 11U> gates{};
        gates.fill(true);
        if (route_index < gates.size()) gates[route_index] = false;
        const bool certified = route_index == 11U;
        const bool selected_closure = route_index == 12U;
        const std::string observed = al_r50_route(
            gates[0], gates[1], gates[2], gates[3], gates[4], gates[5],
            gates[6], gates[7], gates[8], gates[9], gates[10], certified,
            selected_closure);
        routes_exact = routes_exact && observed == expected_routes[route_index];
        route_material << expected_routes[route_index] << ':' << observed << '|';
    }
    const std::string route_root = sha256_hex(route_material.str());
    const std::string route = al_r50_route(parent_exact, source_exact,
        workspace_exact, geometry_exact, capacity_exact, gram_exact,
        hildreth_exact, projection_exact, globalization_exact, primal_exact,
        work_exact && rollback, generator.certified, closure);
    const bool passed = identity && parent_exact && source_exact && dense
        && workspace_exact && geometry_exact && generator_exact
        && capacity_exact && gram_exact && hildreth_exact && projection_exact
        && globalization_exact && primal_exact && work_exact && rollback
        && routes_exact;

    std::ostringstream cache_material;
    for (const ALR50RowBasis& basis : generator.cache)
        cache_material << basis.root << ':';
    const std::string cache_root = sha256_hex(cache_material.str());
    std::ostringstream outer_material;
    for (const ALR50OuterRecord& record : generator.records)
        outer_material << record.root << ':';
    const std::string outer_root = sha256_hex(outer_material.str());
    const std::string witness_root = al_binary64_vec3_root(generator.witness);
    std::ostringstream semantic;
    semantic << std::setprecision(17) << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R50_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << generator.source.primal.psi << ':'
             << generator.source.primal.h << ':'
             << generator.source.primal.maximum_upper << ':'
             << generator.source.active.size() << ':'
             << generator.source.active_root << '|'
             << generator.accepted_outers << ':' << generator.total_sweeps
             << ':' << generator.cache.size() << ':' << cache_root << ':'
             << outer_root << '|' << generator.terminal.primal.psi << ':'
             << generator.terminal.primal.h << ':'
             << generator.terminal.primal.maximum_upper << ':'
             << generator.terminal.active.size() << ':'
             << generator.witness_norm << ':' << witness_root << '|'
             << generator.row_vjp << ':' << generator.gram_jvp << ':'
             << generator.audit_jvp << ':' << generator.pair_passes << ':'
             << generator.certified << ':' << closure << ':' << work_exact
             << ':' << rollback << ':' << route_root << ':' << route;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r19r50_active_face_hildreth_closure.v1\",\"status\":\""
           << (passed ? "PASS" : "FAIL") << "\",\"identity_sha256\":\""
           << B4E2D7R19R50_IDENTITY_SHA256
           << "\",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"source\":{\"psi\":" << generator.source.primal.psi
           << ",\"h\":" << generator.source.primal.h
           << ",\"maximum_upper\":"
           << generator.source.primal.maximum_upper
           << ",\"active\":" << generator.source.active.size()
           << ",\"active_root\":\"" << generator.source.active_root << "\"}"
           << ",\"closure\":{\"accepted_outers\":"
           << generator.accepted_outers
           << ",\"total_sweeps\":" << generator.total_sweeps
           << ",\"cache_rows\":" << generator.cache.size()
           << ",\"cache_root\":\"" << cache_root
           << "\",\"outer_root\":\"" << outer_root
           << "\",\"projection_calls\":" << generator.projection_calls
           << ",\"projection_scans\":" << generator.projection_scans << "}"
           << ",\"primal\":{\"psi\":" << generator.terminal.primal.psi
           << ",\"h\":" << generator.terminal.primal.h
           << ",\"maximum_upper\":"
           << generator.terminal.primal.maximum_upper
           << ",\"raw_active\":" << generator.terminal.primal.raw_active
           << ",\"certified_positive\":"
           << generator.terminal.primal.certified_positive
           << ",\"witness_norm\":" << generator.witness_norm
           << ",\"witness_root\":\"" << witness_root
           << "\",\"certified\":"
           << (generator.certified ? "true" : "false") << "}"
           << ",\"work\":{\"row_vjp\":" << generator.row_vjp
           << ",\"gram_jvp\":" << generator.gram_jvp
           << ",\"audit_jvp\":" << generator.audit_jvp
           << ",\"pair_passes\":" << generator.pair_passes
           << ",\"exact\":" << (work_exact ? "true" : "false") << "}"
           << ",\"dense_root\":\"" << dense_root << "\""
           << ",\"route_cases\":" << expected_routes.size()
           << ",\"route_root\":\"" << route_root
           << "\",\"routes_exact\":" << (routes_exact ? "true" : "false")
           << ",\"rollback_exact\":" << (rollback ? "true" : "false")
           << ",\"route\":\"" << route
           << "\",\"normal_witness_applied\":false"
              ",\"r43_restoration_committed\":false"
              ",\"filter_runtime_committed\":false"
              ",\"restoration_exit_authorized\":false"
              ",\"following_outer_executed\":false"
              ",\"timing_admitted\":false,\"runtime_authority\":false"
              ",\"production_authority\":false,\"result_sha256\":\""
           << result_sha << "\"}";
    if (replay_capture != nullptr) {
        replay_capture->captured = passed;
        replay_capture->r49 = std::move(capture);
        replay_capture->generator = std::move(generator);
        replay_capture->cache_root = cache_root;
        replay_capture->outer_root = outer_root;
        replay_capture->witness_root = witness_root;
    }
    return {passed, report.str()};
}

SplitBoundaryReport run_al_active_face_closure_controls() {
    return run_al_active_face_closure_controls_impl(nullptr);
}
