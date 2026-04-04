# Patch Management

In `n34`, patch management is designed to give you complete control. You can
manually generate patch files using `git-format-patch` and then broadcast them
to Nostr relays. This ensures that you have full authority over the content
and structure of your patches, allowing for precise customization as per your
requirements.

Similarly, when fetching patches, `n34` provides them to you without
automatically applying, merging, or checking them. This empowers you to review
the patches at your own pace and decide whether to merge or apply them as
needed. You retain full control over the entire process, ensuring a tailored
approach to patch management.

## Patch Status Management

You can assign a status to original patches, but revision patches do not have
a specific status assigned to them. Instead, they inherit the status of the
original patch. However, if the original patch is marked as `Applied/Merged`,
the revision patch must be explicitly tagged to claim the same status. If not
tagged, the revision patch status will be `Closed`.
