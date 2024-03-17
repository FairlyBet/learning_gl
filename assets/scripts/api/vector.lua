---@meta _

---@class Vec3
---@field public x number
---@field public y number
---@field public z number
---@operator add(Vec3):Vec3
---@operator sub(Vec3):Vec3
---@operator unm(Vec3):Vec3
---@operator mul(number):Vec3
Vec3 = {}

---@return Vec3
function Vec3.zeros() end

---@param x number
---@param y number
---@param z number
---@return Vec3
function Vec3.new(x, y, z) end

---@return Vec3
function Vec3:normalize() end

---@param self Vec3
---@return string
function Vec3.__tostring(self) end
