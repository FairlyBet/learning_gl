---@class Vector
---@field public x number
---@field public y number
---@field public z number
---@field private __index table
Vector = {}
Vector.__index = Vector

---@return Vector
function Vector:zeroes()
    return Vector:new(0, 0, 0)
end

---@return Vector
function Vector:fromNum(n)
    return Vector:new(n, n, n)
end

---@param x number
---@param y number
---@param z number
---@return Vector
function Vector:new(x, y, z)
    local v = { x = x, y = y, z = z }
    setmetatable(v, self)
    return v
end

---@param num number
---@return Vector
function Vector:addNum(num)
    return Vector:new(self.x + num, self.y + num, self.z + num)
end

---@param v Vector
---@return Vector
function Vector:__add(v)
    return Vector:new(self.x + v.x, self.y + v.y, self.z + v.z)
end

---@param v Vector
---@return Vector
function Vector:__sub(v)
    return self + -v
end

---@return Vector
function Vector:__unm()
    return Vector:new(-self.x, -self.y, -self.z)
end

---@param num number
---@return Vector
function Vector:__mul(num)
    return Vector:new(self.x * num, self.y * num, self.z * num)
end

---@return string
function Vector:__tostring()
    return "X: " .. self.x .. " Y: " .. self.y .. " Z: " .. self.z
end
