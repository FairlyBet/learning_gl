Vector = {}

function Vector:zeroes()
    return Vector:new(0, 0, 0)
end

function Vector:fromNum(n)
    return Vector:new(n, n, n)
end

function Vector:new(x, y, z)
    local v = { x = x, y = y, z = z }
    setmetatable(v, self)
    return v
end

function Vector:__add(v)
    return Vector:new(self.x + v.x, self.y + v.y, self.z + v.z)
end

function Vector:__sub(v)
    return self + -v
end

function Vector:__unm()
    return Vector:new(-self.x, -self.y, -self.z)
end

function Vector.__index()
    return 0
end

function Vector:__tostring()
    return "X: " .. self.x .. " Y: " .. self.y .. " Z: " .. self.z
end
