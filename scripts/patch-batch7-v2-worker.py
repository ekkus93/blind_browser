#!/usr/bin/env python3
from pathlib import Path

path = Path('.github/workflows/batch7-privacy-boundary-v2.yml')
text = path.read_text()
start_marker = '          old_remote_import = '
end_marker = '          remote_path.write_text(remote.replace(old_remote_import, new_remote_import, 1))\n'

if text.count(start_marker) != 1:
    raise SystemExit(f'expected one matcher start, found {text.count(start_marker)}')
if text.count(end_marker) != 1:
    raise SystemExit(f'expected one matcher end, found {text.count(end_marker)}')

start = text.index(start_marker)
end = text.index(end_marker, start) + len(end_marker)
replacement = '''          import re

          remote_import_pattern = re.compile(
              r'(#\\[cfg\\(feature = "remote-openai"\\)\\]\\s*use crate::commands::\\{)'
              r'(?P<body>[^}]*canonical_planner_output_examples[^}]*tool_input_schema[^}]*)'
              r'(\\};)',
              re.S,
          )
          matches = list(remote_import_pattern.finditer(remote))
          if len(matches) != 1:
              raise SystemExit(
                  f"generated remote planner schema import block count={len(matches)}"
              )
          match = matches[0]
          body = match.group("body")
          if "planner_output_schema" in body:
              raise SystemExit("generated remote planner schema import unexpectedly already contains planner_output_schema")
          if body.count("canonical_planner_output_examples") != 1:
              raise SystemExit("generated remote planner canonical example import count was not one")
          body = body.replace(
              "canonical_planner_output_examples",
              "canonical_planner_output_examples, planner_output_schema",
              1,
          )
          remote = remote[: match.start("body")] + body + remote[match.end("body") :]
          remote_path.write_text(remote)
'''

path.write_text(text[:start] + replacement + text[end:])
print('Patched Batch 7 V2 generated import matcher')
