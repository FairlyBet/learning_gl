local object = {}
object.mt = {}

function object.getTransform()

end

local object1 = {}

setmetatable(object1, object)
print(getmetatable(object1))
