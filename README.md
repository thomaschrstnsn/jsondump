# jsondump

Small utility for reading a JSON-file as produced by Azure Data Studio, where the content (one of the columns in the result), is also JSON.

This program can write each row into its own `.json`-file where the name is specified as either:

 - `array-index`: the index of the row in the result set (starting from 0), e.g. `01.json` etc
 - `original-field Id`: A field in the original row, here `Id`, e.g. `1001000.json` etc
 - `nested-field AggregateId`:   A field in the nested json object, here `AggregateId`, e.g. `my-aggregate-123456.json` etc.


Examples, all assume input is in `Results.json`:

```sh
# nested JSON is in column MessageData, create files based on index
jsondump --filename Results.json --jsonfield MessageData array-index
# nested JSON is in column JsonBlob, create files named after the column Id
jsondump --filename Results.json --jsonfield JsonBlob    original-field Id
# nested JSON is in column Data, create files named after the data's own field MessageId
jsondump --filename Results.json --jsonfield Data        nested-field MessageId  
# dry-run (dont actually write any files, just print what would be done)
jsondump --filename Results.json --jsonfield Data --dry-run  nested-field MessageId  
```

The last command could output the following:

```
Simulating an actual run, without writing to files:
Would have written 723 bytes to file: 15.json
Would have written 730 bytes to file: 16.json
Would have written 729 bytes to file: 17.json
```

## Toy project

This was written mostly to get experience with Rust. You are most likely better off using [`jq`](https://jqlang.github.io/jq/) and scripts to do this sort of work.
