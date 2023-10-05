## Changelog

### v2.2.5 (5/10/23)

- Upgrade dependencies

### v2.2.4 (23/7/23)

- Fix rolling averages being offset from the raw data

### v2.2.3 (17/7/23)

- `mongodb_uri` can now be the URI itself instead of just an environment variable

### v2.2.2 (28/5/23)

- Make rolling average calculation faster by capping amount of points calculated
- Fix category colours not being the same over several rolling averages

### v2.2.1 (28/5/23)

- Vendor openssl

### v2.2.0 (28/5/23)

- Change internals of server a lot
- `Loading...` now shows when retrieving data
- Rolling averages and player active times are now processed server-side
- Remove chart animations to make the chart faster

### v2.1.0 (14/11/22)

- Togglable visiblility of rolling averages
- Fixed some leavings not being detected when an Abs record is processed
- Rolling average is now cached
- Added changelog

### v2.0.0 (10/11/22)

- Initial v2 release
