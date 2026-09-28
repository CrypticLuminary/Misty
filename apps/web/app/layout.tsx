import type { ReactNode } from "react";

export const metadata = {
  title: "Misty",
  description: "Original-quality shared memories.",
};

export default function RootLayout({ children }: Readonly<{ children: ReactNode }>) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
