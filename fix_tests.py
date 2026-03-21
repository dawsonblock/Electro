import os, glob

def process_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    # Replace assert!(!tool.declarations().shell_access);
    content = content.replace('assert!(!tool.declarations().shell_access);',
                              'use electro_core::policy::ShellPolicy;\n        assert_eq!(tool.declarations().shell_access, ShellPolicy::Blocked);')
    content = content.replace('assert!(tool.declarations().shell_access);',
                              'use electro_core::policy::ShellPolicy;\n        assert_eq!(tool.declarations().shell_access, ShellPolicy::Allowed);')
    
    with open(filepath, 'w') as f:
        f.write(content)

for root, _, files in os.walk('crates/electro-tools'):
    for file in files:
        if file.endswith('.rs'):
            process_file(os.path.join(root, file))

