import os, glob, re

def process_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    # Replace import
    content = content.replace('use electro_core::{Tool, ToolContext, ToolDeclarations, ToolInput, ToolOutput};',
                              'use electro_core::{Tool, ToolContext, ToolInput, ToolOutput};\nuse electro_core::policy::CapabilityPolicy;\n')
    content = content.replace('use electro_core::{Memory, Tool, ToolContext, ToolDeclarations, ToolInput, ToolOutput};',
                              'use electro_core::{Memory, Tool, ToolContext, ToolInput, ToolOutput};\nuse electro_core::policy::CapabilityPolicy;\n')
    content = content.replace('use electro_core::{PathAccess, Tool, ToolContext, ToolDeclarations, ToolInput, ToolOutput};',
                              'use electro_core::{Tool, ToolContext, ToolInput, ToolOutput};\nuse electro_core::policy::{CapabilityPolicy, FileAccessPolicy};\n')
    content = content.replace('PathAccess, Tool, ToolContext, ToolDeclarations, ToolInput, ToolOutput, ToolOutputImage, Vault',
                              'Tool, ToolContext, ToolInput, ToolOutput, ToolOutputImage, Vault};\nuse electro_core::policy::{CapabilityPolicy, FileAccessPolicy, BrowserPolicy')
    content = content.replace('Channel, PathAccess, Tool, ToolContext, ToolDeclarations, ToolInput, ToolOutput',
                              'Channel, Tool, ToolContext, ToolInput, ToolOutput};\nuse electro_core::policy::{CapabilityPolicy, FileAccessPolicy')
    content = content.replace('use electro_core::{Tool, ToolContext, ToolDeclarations, ToolInput, ToolOutput, UsageStore};',
                              'use electro_core::{Tool, ToolContext, ToolInput, ToolOutput, UsageStore};\nuse electro_core::policy::CapabilityPolicy;\n')
    content = content.replace('use electro_core::{SetupLinkGenerator, Tool, ToolContext, ToolDeclarations, ToolInput, ToolOutput};',
                              'use electro_core::{SetupLinkGenerator, Tool, ToolContext, ToolInput, ToolOutput};\nuse electro_core::policy::CapabilityPolicy;\n')



    content = content.replace('-> ToolDeclarations {', '-> CapabilityPolicy {')
    content = content.replace('ToolDeclarations {', 'CapabilityPolicy {')
    content = content.replace('PathAccess', 'FileAccessPolicy')
    
    # Replace shell_access
    content = re.sub(r'shell_access:\s*false', r'shell_access: electro_core::policy::ShellPolicy::Blocked,\nbrowser_access: electro_core::policy::BrowserPolicy::Blocked', content)
    content = re.sub(r'shell_access:\s*true', r'shell_access: electro_core::policy::ShellPolicy::Allowed,\nbrowser_access: electro_core::policy::BrowserPolicy::Blocked', content)
    
    with open(filepath, 'w') as f:
        f.write(content)

for root, _, files in os.walk('crates'):
    for file in files:
        if file.endswith('.rs'):
            process_file(os.path.join(root, file))

