# track_selector_wasm

To run the project, just run 'npm install' and then 'npm start' and go to `http://localhost:1234` in your browser to view the Demo

## Instructions
To use the application, you can select one of the three given manifests listed from `Select Manifest` dropdown. That should poplate the input field and you can click the `Parse` button to load the file. The `Content Types` area should appear. You first have to select a content type from the `Select Content Type` dropwdown before selecting a bandwidth from the `Select Bandwidth` dropdown. Set a positon and then click the `Get Media Url` button. This will prompt an alert window giving the user a media url of the requrested segment. You should be able to set your own manifet url (only `SegmentTemplate` support for this demp) in the `MPD Url` input field and parse that.

## Note
- There is no error handling in the javsscript application so it may be a buggy... 😬 
- The `SegmentTemplate - MP` asset is a little wierd. Becarful when setting a position with a selected bandwidth. There are different bandwidths in the both Periods so you will get an `undefined` if you search for a position with the worng bandwidth

