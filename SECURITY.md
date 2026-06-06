rStartup Security Policy

We take the security of rStartup seriously. If you discover a vulnerability, please report it following this policy.

Reporting a Vulnerability

Please do not open a public issue for security-related bugs.

Instead, please report security vulnerabilities privately by emailing the maintainers or project leads directly.

We will acknowledge receipt of your report within 48 hours and work with you to analyze and resolve the issue before making any public disclosure.

Scope and Philosophy

Registry and System Access: rStartup reads system metrics and configurations using standard APIs or winreg under Windows. It does not execute raw command strings or start shell contexts with user input, protecting the host system from injection vectors.

Minimal Permissions: rStartup does not require Administrator or UAC elevation to run.

