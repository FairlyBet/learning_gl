local entity = {}
entity.value = {}
entity[entity] = 0

local weakRefMetatable = {}
weakRefMetatable.__index = entity
weakRefMetatable.__metatable = "this metatable is private"
weakRefMetatable.__mode = "v"
setmetatable(weakRefMetatable, weakRefMetatable)
local weakRef = {}
setmetatable(weakRef, weakRefMetatable)

Entities = {}
Entities[0] = { object = entity, weakRef = weakRef }

WeakRefCopy = Entities[0].weakRef

print(WeakRefCopy.value)
print(getmetatable(WeakRefCopy))

Entities[0] = nil

print "After nil"
print(WeakRefCopy.value)
print(getmetatable(WeakRefCopy))
