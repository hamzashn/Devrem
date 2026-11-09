import Link from "next/link";
import type { Dictionary, Locale } from "@/lib/i18n";

export function Header({ locale, t }: { locale: Locale; t: Dictionary }) {
  const other = locale === "tr" ? "en" : "tr";
  const links = [
    { href: `/${locale}/#features`, label: t.nav.features },
    { href: `/${locale}/docs/`, label: t.nav.docs },
    { href: `/${locale}/changelog/`, label: t.nav.changelog },
  ];
  return (
    <header className="sticky top-0 z-30 border-b border-border/70 bg-base/80 backdrop-blur-md">
      <div className="mx-auto flex h-14 max-w-6xl items-center gap-6 px-5">
        <Link href={`/${locale}/`} className="flex items-center gap-2">
          <img src="/logo.svg" alt="" width={26} height={26} />
          <span className="text-[15px] font-semibold tracking-tight">Devrem</span>
        </Link>
        <nav className="hidden items-center gap-5 text-sm text-secondary sm:flex">
          {links.map((l) => (
            <Link key={l.href} href={l.href} className="transition-colors hover:text-primary">
              {l.label}
            </Link>
          ))}
        </nav>
        <div className="ml-auto flex items-center gap-3">
          <Link
            href={`/${other}/`}
            className="rounded-md border border-border px-2 py-1 font-mono text-[11px] uppercase text-secondary transition-colors hover:text-primary"
            hrefLang={other}
          >
            {other}
          </Link>
          <Link
            href={`/${locale}/download/`}
            className="rounded-md bg-accent px-3.5 py-1.5 text-sm font-medium text-[var(--bg-base)] shadow-[0_0_18px_var(--accent-glow)] transition hover:brightness-110"
          >
            {t.nav.download}
          </Link>
        </div>
      </div>
    </header>
  );
}
