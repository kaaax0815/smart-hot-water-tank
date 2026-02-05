-- name: DevSeedLogin :exec
INSERT INTO logins (name, api_key)
VALUES ('Test User', '9c2320fa97aa6a17b35e01db9bcc0b251898dc5a95178bbb0ee1dde4dc4a7accba8f7746573a6869f68554f07d2ac2885d4b4b0afe9f544d474f31b89ff6ccbb');

-- name: GetLoginByApiKey :one
UPDATE logins
SET used_at = NOW()
WHERE api_key = $1
RETURNING id, api_key;