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
