## dev server

Simple dev http proxy config

---

- `git.jaemk.me` -> `https://github.com/jaemk`
- `git.jaemk.me/cached` -> `https://github.com/jaemk/cached`
- `status.jaemk.me` -> check site /status endpoints
- `jaemk.me` -> local dev server at `localhost:3003`

---

**Generate cert**

```
./certs.sh new

./certs.sh renew
```
