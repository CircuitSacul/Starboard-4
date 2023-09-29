#!/usr/local/bin/zx

let dest;
try {
    dest = (await $` echo $BK_DEST`).stdout
} catch {
    dest = undefined
}

if (dest !== undefined && dest.trim() == "") {
    dest = undefined
}

await cd('/backups')
await $`mkdir -p backups`
await $`mkdir -p temp`
await cd('backups')
await $`touch phantom.daily phantom.hourly`
await cd('..')
await $`mv backups/* temp/`
await cd('backups')

let hour = (await $`date +"%H"`).stdout.trim()
let isDailyBackup = hour === "00"

// e.g. 2022-01-25.hour-11.sql.gz
// e.g. 2022-01-25.daily.sql.gz
let fileName = (await $`date +"%F"`).stdout.trim() + (isDailyBackup ? ".daily" : `.${hour}.hourly`)

await $`pg_dump -d $SB_DATABASE_URL -Z 9 -f ${fileName}`

await cd('../temp')
if (isDailyBackup) {
    await $`rm *.daily`
    await $`mv *.hourly ../backups`

    await cd("../..")
    if (dest !== undefined) {
        await $` sshpass -p "$RSYNC_PASSWORD" rsync --delete-after -rt backups/backups/ $BK_DEST -e "ssh -o StrictHostKeyChecking=no"`
    }
} else {
    await $`rm *.hourly`
    await $`mv *.daily ../backups`
}
