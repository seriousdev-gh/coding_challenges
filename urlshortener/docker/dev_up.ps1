$env:SERVICE_HOST = 'http://localhost'
$exe = "docker"
&$exe compose up
Remove-Item Env:\SERVICE_HOST