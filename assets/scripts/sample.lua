-- local function CreateContext(ptr)

-- end

-- GLOBAL_OBJECT = { 10 }

-- local copy = GLOBAL_OBJECT
-- GLOBAL_OBJECT = nil
-- print(copy[1])

-- local engine = {}
-- function engine.GetInput()

-- end

-- local balance = {}

-- Account = {}

-- function Account:withdraw(v)
--     balance[self] = balance[self] - v
-- end

-- function Account:deposit(v)
--     balance[self] = balance[self] + v
-- end

-- function Account:balance()
--     return balance[self]
-- end

-- function Account:new(o)
--     o = o or {} -- create table if user does not provide one
--     setmetatable(o, self)
--     self.__index = self
--     balance[o] = 0 -- initial balance
--     return o
-- end


local function setAddress(address, fun)
    return function(id)
        fun(address, id)
    end
end

function GetTransform(address, id)
end

GetTransform = setAddress(0x0012312abc4041, GetTransform)


GetTransform(228)
