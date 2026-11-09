import Link from "next/link";
import type { Dictionary, Locale } from "@/lib/i18n";
import { LINKS } from "@/lib/site";

export function Footer({ locale, t }: { locale: Locale; t: Dictionary }) {
  return (
    <footer className="mt-24 border-t border-border">
      <div className="mx-auto flex max-w-6xl flex-col gap-4 px-5 py-8 text-sm text-secondary sm:flex-row sm:items-center">
        <div className="flex items-center gap-2">
          <img src="/logo.svg" alt="" width={18} height={18} />
          <span>{t.footer.rights}</span>
        </div>
        <span className="sm:ml-4">{t.footer.madeWith}</span>
        <nav className="flex gap-4 sm:ml-auto">
          <Link href={`/${locale}/docs/`} className="hover:text-primary">
            {t.nav.docs}
          </Link>
          <Link href={`/${locale}/changelog/`} className="hover:text-primary">
            {t.nav.changelog}
          </Link>
          <a href={LINKS.releases} className="hover:text-primary">
            GitHub
          </a>
        </nav>
      </div>
    </footer>
  );
}
