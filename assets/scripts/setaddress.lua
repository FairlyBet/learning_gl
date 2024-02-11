return function(address, func)
    return function(arg)
        rawset(arg, "address", address)
        func(address, arg)
    end
end
