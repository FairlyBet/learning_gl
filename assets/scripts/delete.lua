Delete = {}
Delete.__index = Delete

function Delete:update()
    if Input.getKey(Keys.Delete, Actions.Press) then
        -- DeleteScript(self)
    end
end

local delete = {}
setmetatable(delete, Delete)

return delete
