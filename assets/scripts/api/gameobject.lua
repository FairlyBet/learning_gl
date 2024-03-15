Object = {}
Object.value = 10
Object[Object] = 0

WeakRefMetatable = {}
WeakRefMetatable.__index = Object
WeakRefMetatable.__metatable = "the metatable is private"
WeakRefMetatable.__mode = "v"
setmetatable(WeakRefMetatable, WeakRefMetatable)
WeakRef = {}
setmetatable(WeakRef, WeakRefMetatable)

print(WeakRef.value)
print(getmetatable(WeakRef))
Object = nil

---@class Entity
---@field public transform Transform
Entity = {}
