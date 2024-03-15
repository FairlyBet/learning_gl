---@meta _
---@class Transform
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

---@param component table
---@return Vec3
function Transform.getGlobalPosition(component) end

---@param component table
---@param position Vec3
function Transform.setPosition(component, position) end

---@param component table
---@param euler Vec3
function Transform.rotate(component, euler) end

---@param component table
---@param euler Vec3
function Transform.rotateLocal(component, euler) end

---@param component table
---@return Vec3
function Transform.getOrientation(component) end

---@param component table
---@param position Vec3
function Transform.setOrientation(component, position) end
