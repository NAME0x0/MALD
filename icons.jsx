// Simple inline SVG icons matching Lucide style — geometric, 1.5px stroke
const Icon = ({ children, size = 12, color = 'currentColor', strokeWidth = 1.5, ...rest }) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 24 24"
    fill="none"
    stroke={color}
    strokeWidth={strokeWidth}
    strokeLinecap="round"
    strokeLinejoin="round"
    style={{ display: 'block', flexShrink: 0 }}
    {...rest}
  >
    {children}
  </svg>
);

const IconFileText = (p) => (
  <Icon {...p}>
    <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
    <polyline points="14 2 14 8 20 8" />
    <line x1="8" y1="13" x2="16" y2="13" />
    <line x1="8" y1="17" x2="16" y2="17" />
  </Icon>
);
const IconSparkles = (p) => (
  <Icon {...p}>
    <path d="M12 3l1.7 4.6L18 9l-4.3 1.4L12 15l-1.7-4.6L6 9l4.3-1.4z" />
    <path d="M19 15l.8 2.2L22 18l-2.2.8L19 21l-.8-2.2L16 18l2.2-.8z" />
  </Icon>
);
const IconShield = (p) => (
  <Icon {...p}>
    <path d="M12 2l8 3v6c0 5-3.5 9-8 11-4.5-2-8-6-8-11V5z" />
  </Icon>
);
const IconZap = (p) => (
  <Icon {...p}>
    <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2" />
  </Icon>
);
const IconLink = (p) => (
  <Icon {...p}>
    <path d="M10 13a5 5 0 0 0 7 0l3-3a5 5 0 0 0-7-7l-1 1" />
    <path d="M14 11a5 5 0 0 0-7 0l-3 3a5 5 0 0 0 7 7l1-1" />
  </Icon>
);
const IconGitBranch = (p) => (
  <Icon {...p}>
    <line x1="6" y1="3" x2="6" y2="15" />
    <circle cx="18" cy="6" r="3" />
    <circle cx="6" cy="18" r="3" />
    <path d="M18 9a9 9 0 0 1-9 9" />
  </Icon>
);
const IconQuote = (p) => (
  <Icon {...p}>
    <path d="M21 15a2 2 0 0 1-2 2h-7l-4 4v-4H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h15a2 2 0 0 1 2 2z" />
    <path d="M8 9v2" />
    <path d="M14 9v2" />
  </Icon>
);
const IconLayers = (p) => (
  <Icon {...p}>
    <polygon points="12 2 2 7 12 12 22 7 12 2" />
    <polyline points="2 17 12 22 22 17" />
    <polyline points="2 12 12 17 22 12" />
  </Icon>
);
const IconActivity = (p) => (
  <Icon {...p}>
    <polyline points="22 12 18 12 15 21 9 3 6 12 2 12" />
  </Icon>
);
const IconCopy = (p) => (
  <Icon {...p}>
    <rect x="9" y="9" width="13" height="13" rx="2" />
    <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
  </Icon>
);
const IconCheck = (p) => (
  <Icon {...p}>
    <polyline points="20 6 9 17 4 12" />
  </Icon>
);
const IconArrowDown = (p) => (
  <Icon {...p}>
    <line x1="12" y1="5" x2="12" y2="19" />
    <polyline points="19 12 12 19 5 12" />
  </Icon>
);
const IconDiamond = (p) => (
  <Icon {...p}>
    <path d="M12 2 L22 12 L12 22 L2 12 Z" />
  </Icon>
);

Object.assign(window, {
  Icon, IconFileText, IconSparkles, IconShield, IconZap, IconLink,
  IconGitBranch, IconQuote, IconLayers, IconActivity, IconCopy, IconCheck,
  IconArrowDown, IconDiamond,
});
