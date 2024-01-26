- Login: `POST /auth/token`

```python
import requests

url = "http://localhost:7999/auth/token"

# option values: web, apple, platform, default: web
payload = 'username=admin%40gmail.com&password=admin&option=web'
headers = {
  'Content-Type': 'application/x-www-form-urlencoded'
}

response = requests.request("POST", url, headers=headers, data=payload)

print(response.text)
```

- Refresh `RefreshToken`: `POST /auth/refresh_token`

``` python
import requests

url = "http://localhost:7999/auth/refresh_token"

payload = {}
headers = {
  'Authorization': 'Bearer your_refresh_token'
}

response = requests.request("POST", url, headers=headers, data=payload)

print(response.text)

```

- Revoke `RefreshToken`: `POST /auth/revoke_token`

```python
import requests

url = "http://localhost:7999/auth/revoke_token"

payload = {}
headers = {
  'Authorization': 'Bearer your_refresh_token'
}

response = requests.request("POST", url, headers=headers, data=payload)

print(response.text)

```

- Refresh `Session`: `POST /auth/refresh_session`

```python
import requests

url = "http://localhost:7999/auth/refresh_session"

payload = {}
headers = {
  'Authorization': 'Bearer your_refresh_session'
}

response = requests.request("POST", url, headers=headers, data=payload)

print(response.text)

```

- Obtain `Sess token`: `POST /auth/sess_token`

```python
import requests

url = "http://localhost:7999/auth/sess_token"

payload = {}
headers = {
  'Authorization': 'Bearer your_platform_access_token'
}

response = requests.request("POST", url, headers=headers, data=payload)

print(response.text)

```

- Obtain `Billing`: `GET /auth/billing`

```python
import requests

url = "http://localhost:7999/auth/billing"

payload = {}
headers = {
  'Authorization': 'Bearer your_sess_token'
}

response = requests.request("POST", url, headers=headers, data=payload)

print(response.text)

```
