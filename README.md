# List unsubscribe

Small tool that will mass unsubscribe from mailing lists.

## How it works

This tool will scan emails and look for the `List-Unsubscribe` header and will
try to unsubscribe by sending emails to the provided address and also follow
`One-Click` unsubscribe link.

## Usage

To use, put emails you want to unsubscribe from in a folder, and run the tool
like this:


```shell
list-unsubscribe -d /path/to/folder -f default_sender@example.com \
  -s your-email-server.example.com -n "Sender Name"
```

The tool will use port 587 without authentication to send email. If you need
authentication, pull requests are welcome.
