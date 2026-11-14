
import { useState, type FormEvent } from "react";
import { useNavigate } from "react-router";
import { api, ApiError } from "../api/client";

export function AuthPage() {
  const navigate = useNavigate();

  const [registerMode, setRegisterMode] = useState(false);
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  async function handleSubmit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setLoading(true);
    setError("");

    try {
      const credentials = { username, password };

      if (registerMode) {
        await api.register(credentials);
      } else {
        await api.login(credentials);
      }

      navigate("/inbox", { replace: true });
    } catch (err) {
      setError(
        err instanceof ApiError
          ? err.message
          : "Authentication failed"
      );
    } finally {
      setLoading(false);
    }
  }

  return (
    <div style={{ maxWidth: 400, margin: "48px auto" }}>
      <h1>{registerMode ? "Register" : "Login"}</h1>

      <form
        onSubmit={handleSubmit}
        style={{ display: "flex", flexDirection: "column", gap: 12 }}
>
        <label htmlFor="username">Username</label>
        <input
          id="username"
          value={username}
          onChange={(e) => setUsername(e.target.value)}
          autoComplete="username"
          minLength={3}
          maxLength={32}
          required
        />

        <label htmlFor="password">Password</label>
        <input
          id="password"
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          autoComplete={registerMode ? "new-password" : "current-password"}
          minLength={registerMode ? 12 : undefined}
          required
        />

        {error && <p role="alert">{error}</p>}

        <button type="submit" disabled={loading}>
          {loading ? "Please wait..." : registerMode ? "Register" : "Login"}
        </button>
      </form>

      <button
        type="button"
        onClick={() => {
          setRegisterMode(!registerMode);
          setError("");
        }}
      >
        {registerMode
          ? "Already have an account? Login"
          : "Don't have an account? Register"}
      </button>
    </div>
  );
}
