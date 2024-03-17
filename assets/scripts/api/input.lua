---@meta _

---@class Input
Input = {}

---@param key userdata
---@param action userdata
---@param modifiers? userdata
---@return boolean
function Input.getKey(key, action, modifiers) end

---@param key userdata
---@return boolean
function Input.getKeyHeld(key) end

---@return number, number
function Input.getCursorPosition() end

---@return number, number
function Input.getCursorOffset() end

---@class Keys
---@field Space userdata
---@field Apostrophe userdata
---@field Comma userdata
---@field Minus userdata
---@field Period userdata
---@field Slash userdata
---@field Num0 userdata
---@field Num1 userdata
---@field Num2 userdata
---@field Num3 userdata
---@field Num4 userdata
---@field Num5 userdata
---@field Num6 userdata
---@field Num7 userdata
---@field Num8 userdata
---@field Num9 userdata
---@field Semicolon userdata
---@field Equal userdata
---@field A userdata
---@field B userdata
---@field C userdata
---@field D userdata
---@field E userdata
---@field F userdata
---@field G userdata
---@field H userdata
---@field I userdata
---@field J userdata
---@field K userdata
---@field L userdata
---@field M userdata
---@field N userdata
---@field O userdata
---@field P userdata
---@field Q userdata
---@field R userdata
---@field S userdata
---@field T userdata
---@field U userdata
---@field V userdata
---@field W userdata
---@field X userdata
---@field Y userdata
---@field Z userdata
---@field LeftBracket userdata
---@field Backslash userdata
---@field RightBracket userdata
---@field GraveAccent userdata
---@field World1 userdata
---@field World2 userdata
---@field Escape userdata
---@field Enter userdata
---@field Tab userdata
---@field Backspace userdata
---@field Insert userdata
---@field Delete userdata
---@field Right userdata
---@field Left userdata
---@field Down userdata
---@field Up userdata
---@field PageUp userdata
---@field PageDown userdata
---@field Home userdata
---@field End userdata
---@field CapsLock userdata
---@field ScrollLock userdata
---@field NumLock userdata
---@field PrintScreen userdata
---@field Pause userdata
---@field F1 userdata
---@field F2 userdata
---@field F3 userdata
---@field F4 userdata
---@field F5 userdata
---@field F6 userdata
---@field F7 userdata
---@field F8 userdata
---@field F9 userdata
---@field F10 userdata
---@field F11 userdata
---@field F12 userdata
---@field F13 userdata
---@field F14 userdata
---@field F15 userdata
---@field F16 userdata
---@field F17 userdata
---@field F18 userdata
---@field F19 userdata
---@field F20 userdata
---@field F21 userdata
---@field F22 userdata
---@field F23 userdata
---@field F24 userdata
---@field F25 userdata
---@field Kp0 userdata
---@field Kp1 userdata
---@field Kp2 userdata
---@field Kp3 userdata
---@field Kp4 userdata
---@field Kp5 userdata
---@field Kp6 userdata
---@field Kp7 userdata
---@field Kp8 userdata
---@field Kp9 userdata
---@field KpDecimal userdata
---@field KpDivide userdata
---@field KpMultiply userdata
---@field KpSubtract userdata
---@field KpAdd userdata
---@field KpEnter userdata
---@field KpEqual userdata
---@field LeftShift userdata
---@field LeftControl userdata
---@field LeftAlt userdata
---@field LeftSuper userdata
---@field RightShift userdata
---@field RightControl userdata
---@field RightAlt userdata
---@field RightSuper userdata
---@field Menu userdata
---@field Unknown userdata
Keys = {}

---@class Actions
---@field Press userdata
---@field Release userdata
---@field Repeat userdata
Actions = {}

---@class Modifiers
---@field CapsLock userdata
---@field Control userdata
---@field NumLock userdata
---@field Shift userdata
---@field Super userdata
---@operator bor(userdata):userdata
Modifiers = {}
