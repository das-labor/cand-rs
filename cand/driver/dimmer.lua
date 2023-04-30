register_driver('lap:dimmer', function (device, room, channel, config)
    return {
        flags = ["read", "write", "subscribe"]
        set_channel = function(request, value) 
            can.send_message() -- TODO
            request.reply_ok()
        end,
        get_channel = function(request)
            can.subscribe_once(id, bitmask, message, bitmask, {timeout = 200}, function(result, id, message)
                if result == true then
                    request.reply_channel_value(...)
                else
                    -- timeout
                    request.reply_err(errcode.timeout)
                end
            end)
        end,
        subscribe_channel = function(request)
            can.subscribe(id, bitmask, message, bitmask, function(id, message)
                request.reply_channel_value(...)
            end)
            request.reply_ok()
        end,
    }
end)