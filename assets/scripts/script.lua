local object = {}
print(tostring(object))
local table = {}
object[table] = 12

for key, value in pairs(object) do
    print("Key: " .. tostring(key) .. "\t\tValue: " .. tostring(value))
end
