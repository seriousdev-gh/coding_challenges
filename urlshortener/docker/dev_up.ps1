$env:SERVICE_HOST = 'http://localhost'
$exe = "docker"
&$exe compose up --build
Remove-Item Env:\SERVICE_HOST