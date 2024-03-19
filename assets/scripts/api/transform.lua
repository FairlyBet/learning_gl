---@meta _
---@class Transform
Transform = {}

---@param entity Entity
---@param delta Vec3
function Transform.move(entity, delta) end

---@param entity Entity
---@param delta Vec3
function Transform.moveLocal(entity, delta) end

---@param entity Entity
---@return Vec3
function Transform.getPosition(entity) end

---@param entity Entity
---@return Vec3
function Transform.getGlobalPosition(entity) end

---@param entity Entity
---@param position Vec3
function Transform.setPosition(entity, position) end

---@param entity Entity
---@param euler Vec3
function Transform.rotate(entity, euler) end

---@param entity Entity
---@param euler Vec3
function Transform.rotateLocal(entity, euler) end

---@param entity Entity
---@return Vec3
function Transform.getOrientation(entity) end

---@param entity Entity
---@param position Vec3
function Transform.setOrientation(entity, position) end
