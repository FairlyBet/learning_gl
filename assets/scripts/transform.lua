---@meta _

Transform = {}

---@param component table
---@param delta Vec3
function Transform.move(component, delta) end

---@param component table
---@param delta Vec3
function Transform.moveLocal(component, delta) end

---@param component table
---@return Vec3
function Transform.getPosition(component) end
