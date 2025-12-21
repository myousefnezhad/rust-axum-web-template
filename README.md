Rust + Axum Web Template (Auth + Postgres + Redis)
=========================
A clean, production-minded starter template for building web APIs with Rust and Axum.

This template ships with a complete authentication flow based on access tokens and refresh tokens:
	•	Access token: short-lived token used to authorize requests to protected endpoints.
	•	Refresh token: long-lived token used to obtain new access tokens without re-logging in.
	•	Refresh tokens are persisted in Redis (so you can revoke sessions, enforce logout, and support multi-device logins).

It also includes a basic user management API backed by PostgreSQL, with endpoints to:
	•	Register / add users (store user records in Postgres)
	•	Login users (validate credentials, issue access + refresh tokens, save refresh token in Redis)
	•	Remove users (delete user records and optionally invalidate active refresh tokens)

Overall, it’s intended as a simple but solid foundation for Axum services, with a clear separation between routing, handlers, auth middleware, database access (Postgres), and session/token storage (Redis).
