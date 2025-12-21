# VMStuff TODO list
- DONE Add admin API to list users by org
- DONE Simple Authn (before OIDC)
- DONE Authz Add roles: user, admin, associated with user/org link.
- Authz - actually restrict operations based on org/role
- UI
  - Started: Login page only
  - Update JOURNAL with initial UI work
- Compose for running app in production
- Maybe change auth to separate identity from access tokens. Changes the follow on flow... now we always verify identity. Then create authz tokens for the org we want to use.
- OIDC authn

