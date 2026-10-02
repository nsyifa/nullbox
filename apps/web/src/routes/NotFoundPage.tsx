import { Link } from "react-router";
import { useCopy } from "../copy/CopyProvider";

export function NotFoundPage() {
  const copy = useCopy();
  return (
    <section>
      <h1>404</h1>
      <p>{copy.errors.not_found}</p>
      <Link to="/">← {copy.app.nav.submit}</Link>
    </section>
  );
}
