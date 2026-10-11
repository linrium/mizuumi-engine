"use client";

import {
  IconDatabase,
  IconMessageCircle,
  IconPlugConnected,
  IconBuildingBank,
  IconTerminal2,
  IconHome,
  IconShape,
  IconShape2,
  IconIcons,
  IconSmartHome,
  IconCpu,
  IconPipeline,
  IconShield,
  IconGitPullRequest,
  IconChartPpf,
  IconCodeAsterisk,
  IconSparkle2,
} from "@tabler/icons-react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import type { ComponentProps } from "react";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarRail,
} from "@/components/ui/sidebar";
import { SignOutButton } from "@/components/sign-out-button";

type NavItem = {
  href: string;
  label: string;
  icon: React.ComponentType<{ className?: string }>;
};

const navSections: { label: string; items: NavItem[] }[] = [
  {
    label: "Home",
    items: [
      { href: "/", label: "Home", icon: IconSparkle2 },
      { href: "/unitycatalog", label: "Catalog", icon: IconIcons },
      { href: "/computes", label: "Computes", icon: IconCpu },
      { href: "/pipelines", label: "Pipelines", icon: IconPipeline },
      { href: "/guard", label: "Review", icon: IconGitPullRequest },
      { href: "/mcp", label: "MCP", icon: IconPlugConnected },
      { href: "/dashboard", label: "Dashboard", icon: IconChartPpf },
      { href: "/sql", label: "Interactive", icon: IconCodeAsterisk },
    ],
  },
  {
    label: "Data",
    items: [{ href: "/banking", label: "Banking", icon: IconBuildingBank }],
  },
];

type AppSidebarProps = ComponentProps<typeof Sidebar> & {
  user?: { name?: string | null; email?: string | null };
};

export function AppSidebar({ user, ...props }: AppSidebarProps) {
  const pathname = usePathname();

  return (
    <Sidebar collapsible="icon" {...props}>
      <SidebarHeader>
        <div className="flex items-center gap-2 px-2 py-1.5">
          <div className="flex flex-col leading-tight group-data-[collapsible=icon]:hidden">
            <span className="text-sm font-semibold">Mizuumi Studio</span>
          </div>
        </div>
      </SidebarHeader>
      <SidebarContent>
        {navSections.map((section) => (
          <SidebarGroup key={section.label}>
            <SidebarGroupLabel>{section.label}</SidebarGroupLabel>
            <SidebarGroupContent>
              <SidebarMenu>
                {section.items.map((item) => {
                  const isActive =
                    item.href === "/"
                      ? pathname === "/"
                      : pathname === item.href ||
                        pathname.startsWith(`${item.href}/`);
                  const Icon = item.icon;
                  return (
                    <SidebarMenuItem key={item.href}>
                      <SidebarMenuButton
                        render={
                          <Link href={item.href}>
                            <Icon />
                            <span>{item.label}</span>
                          </Link>
                        }
                        isActive={isActive}
                        tooltip={item.label}
                      />
                    </SidebarMenuItem>
                  );
                })}
              </SidebarMenu>
            </SidebarGroupContent>
          </SidebarGroup>
        ))}
      </SidebarContent>
      <SidebarFooter>
        <div className="flex flex-col gap-2 px-2 py-1.5 group-data-[collapsible=icon]:hidden">
          {user ? (
            <div className="flex flex-col leading-tight">
              <span className="truncate text-sm font-medium">
                {user.name ?? "Signed in"}
              </span>
              {user.email ? (
                <span className="truncate text-xs text-muted-foreground">
                  {user.email}
                </span>
              ) : null}
            </div>
          ) : null}
          <SignOutButton />
        </div>
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  );
}
