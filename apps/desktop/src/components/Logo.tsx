export function Logo({ size = 22 }: { size?: number }) {
  return <img src="/logo.svg" width={size} height={size} alt="" draggable={false} />;
}
