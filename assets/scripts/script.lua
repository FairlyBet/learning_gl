ObjectPool = {}

function ObjectPool:__newindex(k, v)

end

function ObjectPool:update()
    self.gameObject.transform.move(Vec3.zeros())
end

function ObjectPool.new()
    local object = {}
    setmetatable(object, ObjectPool)
    return object
end

return ObjectPool.new
