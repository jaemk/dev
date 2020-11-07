## dev server

A simple dev http proxy that should just be an nginx configuration, but I wanted to build something.

---

- `git.jaemk.me` -> `https://github.com/jaemk`
- `git.jaemk.me/cached` -> `https://github.com/jaemk/cached`
- `jaemk.me` -> local dev server at `localhost:3003`

---

**Generate cert**

```
certbot certonly -a webroot --webroot-path $PROJECT_ROOT/static --email $EMAIL -d $DOMAIN
```
