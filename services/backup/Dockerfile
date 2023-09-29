FROM node:20-alpine3.17

RUN apk update
RUN apk add bash
RUN apk add postgresql-client
RUN apk add rsync openssh sshpass

RUN npm i -g zx

COPY services/backup/backup.mjs /usr/bin/backup.mjs
RUN chmod +x /usr/bin/backup.mjs
COPY services/backup/crontab /etc/crontabs/root

CMD ["crond", "-f", "-l", "2"]
