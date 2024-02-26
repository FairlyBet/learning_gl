---@meta _

---@class Vec3
---@field public x number
---@field public y number
---@field public z number
---@field private __index table
Vec3 = {}

---@return Vec3
function Vec3.zeros() end

---@param x number
---@param y number
---@param z number
---@return Vec3
function Vec3.new(x, y, z) end

---@param v Vec3
---@return Vec3
function Vec3:__add(v) end

---@param v Vec3
---@return Vec3
function Vec3:__sub(v) end

---@return Vec3
function Vec3:__unm() end

---@param num number
---@return Vec3
function Vec3:__mul(num) end

---@return string
function Vec3:__tostring() end
