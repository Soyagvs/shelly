$enabled = true
$count = 3
$result = "before"

if $enabled
    $result = "enabled branch"
elif $count >= 10
    $result = "large count"
else
    $result = "fallback"
end

shellout $result

if "hello $result" != ""
    shellout "interpolation works"
end
