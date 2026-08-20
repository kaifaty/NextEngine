
local health = nextengine.query_target_health()
if health > 0 then
    nextengine.propose_action("nextengine.action.melee")
    nextengine.set_state("executed")
end
