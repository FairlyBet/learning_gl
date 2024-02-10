function Readonly(t)
    local proxy = {}
    local mt = {
        __index = t,
        __metatable = "this is readonly table",
        __newindex = function(_, _, _)
            error("attempt to update a read-only table", 2)
        end
    }
    setmetatable(t, proxy)
    return proxy
end
