# Brutalist member directory

Rust, Axum, MongoDB, and a static frontend for sign-up, sign-in, and an authenticated user directory.

## Run locally

1. Copy `.env.example` to `.env` and set the MongoDB connection details and a strong `JWT_SECRET` (32 or more characters).
2. From the project root, run `cargo run`.
3. Open `http://localhost:3000` in your browser. The Rust server serves the frontend and API from the same origin.

## Security and behaviour

- Passwords are hashed with Argon2 and never returned by the API.
- Login creates an HttpOnly, `SameSite=Strict` session cookie that expires after one hour. Set `COOKIE_SECURE=true` when serving over HTTPS in production.
- Emails are trimmed and normalized to lowercase. MongoDB enforces a unique index on `email`.
- Signed-in users can view the paginated name-and-email directory. The API returns 50 users per page, with a **Load more users** button.

## API routes

- `POST /signup` — creates an account and returns `201 Created`.
- `POST /login` — starts a session with an HttpOnly cookie.
- `POST /logout` — clears the session cookie.
- `GET /profile` — returns the signed-in user's name and email.
- `GET /users?page=1&limit=50` — returns a protected, paginated directory.
