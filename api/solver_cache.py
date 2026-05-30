import hashlib
import json
import sqlite3
from pathlib import Path
from typing import Any


DB_PATH = Path(__file__).resolve().parent / "data" / "solver_cache.db"


def _get_cache_key(solver_type: str, iterations: int) -> str:
    raw_key = f"{solver_type}:{iterations}".encode("utf-8")
    return hashlib.sha256(raw_key).hexdigest()


def _connect() -> sqlite3.Connection:
    DB_PATH.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(DB_PATH)
    conn.row_factory = sqlite3.Row
    return conn


def _init_db() -> None:
    with _connect() as conn:
        conn.execute(
            """
            CREATE TABLE IF NOT EXISTS solver_results (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                cache_key TEXT NOT NULL UNIQUE,
                solver_type TEXT NOT NULL,
                iterations INTEGER NOT NULL,
                strategies_json TEXT NOT NULL,
                elapsed_ms REAL NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            """
        )


def _row_to_result(row: sqlite3.Row | None) -> dict[str, Any] | None:
    if row is None:
        return None

    return {
        "id": row["id"],
        "cache_key": row["cache_key"],
        "solver_type": row["solver_type"],
        "iterations": row["iterations"],
        "strategies": json.loads(row["strategies_json"]),
        "elapsed_ms": row["elapsed_ms"],
        "created_at": row["created_at"],
    }


def get(solver_type: str, iterations: int) -> dict[str, Any] | None:
    _init_db()
    cache_key = _get_cache_key(solver_type, iterations)
    with _connect() as conn:
        row = conn.execute(
            """
            SELECT id, cache_key, solver_type, iterations, strategies_json,
                   elapsed_ms, created_at
            FROM solver_results
            WHERE cache_key = ?
            """,
            (cache_key,),
        ).fetchone()
    return _row_to_result(row)


def save(
    solver_type: str,
    iterations: int,
    strategies: dict[str, list[float]],
    elapsed_ms: float,
) -> int:
    _init_db()
    cache_key = _get_cache_key(solver_type, iterations)
    strategies_json = json.dumps(strategies, sort_keys=True)
    with _connect() as conn:
        cursor = conn.execute(
            """
            INSERT INTO solver_results (
                cache_key, solver_type, iterations, strategies_json, elapsed_ms
            )
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(cache_key) DO UPDATE SET
                strategies_json = excluded.strategies_json,
                elapsed_ms = excluded.elapsed_ms
            RETURNING id
            """,
            (cache_key, solver_type, iterations, strategies_json, elapsed_ms),
        )
        return int(cursor.fetchone()["id"])


def list_all() -> list[dict[str, Any]]:
    _init_db()
    with _connect() as conn:
        rows = conn.execute(
            """
            SELECT id, cache_key, solver_type, iterations, strategies_json,
                   elapsed_ms, created_at
            FROM solver_results
            ORDER BY created_at DESC, id DESC
            """
        ).fetchall()
    return [result for row in rows if (result := _row_to_result(row)) is not None]


def get_by_id(result_id: int) -> dict[str, Any] | None:
    _init_db()
    with _connect() as conn:
        row = conn.execute(
            """
            SELECT id, cache_key, solver_type, iterations, strategies_json,
                   elapsed_ms, created_at
            FROM solver_results
            WHERE id = ?
            """,
            (result_id,),
        ).fetchone()
    return _row_to_result(row)


def delete(result_id: int) -> bool:
    _init_db()
    with _connect() as conn:
        cursor = conn.execute("DELETE FROM solver_results WHERE id = ?", (result_id,))
        return cursor.rowcount > 0
