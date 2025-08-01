from itsdangerous import URLSafeSerializer, BadSignature, SignatureExpired
from flask import current_app

def _get_serializer() -> URLSafeSerializer:
    """Creates a serializer using Flask's SECRET_KEY"""
    return URLSafeSerializer(
        secret_key=current_app.config["SECRET_KEY"]
    )

def generate_user_token(user_token: str) -> str:
    """Generates a signed token tied to both the Flask secret and user token."""
    s = _get_serializer()
    return s.dumps(user_token)

def verify_user_token(token: str) -> dict | None:
    """Verifies a signed token using Flask secret and user token as salt."""
    s = _get_serializer()
    try:
        return s.loads(token)
    except (BadSignature, SignatureExpired):
        return None
