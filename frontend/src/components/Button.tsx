import type { ButtonHTMLAttributes } from "react";

type Variant = "primary" | "ghost" | "danger";

const VARIANT_CLASS: Record<Variant, string> = {
  primary: "btn",
  ghost: "btn ghost",
  danger: "btn danger",
};

export function Button({
  variant = "primary",
  className = "",
  ...rest
}: ButtonHTMLAttributes<HTMLButtonElement> & { variant?: Variant }) {
  return <button className={`${VARIANT_CLASS[variant]} ${className}`.trim()} {...rest} />;
}
