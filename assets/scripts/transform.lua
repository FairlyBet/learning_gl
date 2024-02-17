---@meta _

Transform = {}

---@param component table
---@param delta Vector
function Transform.move(component, delta) end

---@param component table
---@param delta Vector
function Transform.moveLocal(component, delta) end

---@param component table
---@return Vector
function Transform.getPosition(component) end
