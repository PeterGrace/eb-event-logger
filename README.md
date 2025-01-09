# What it does
This simple program runs a loop that, every 30 seconds, queries AWS elastic beanstalk to enumerate all beanstalk environments in your logged-in account and then gathers all of the events that were emitted since the last loop processing period and shoves them to a configurable http endpoint.

# Requirements
app must be run in a shell that is logged into AWS, or, in a kubernetes POD that has EKS IAM Roles for Service Accounts (IRSA) enabled.
The following envvars are read for configuration:
| var | what |
| --- | ---- |
| AWS_* | Whichever AWS auth envvars your deployment uses |
| EB_EVENT_LOGGER_URL | The URL for your http receiver. |
| EB_EVENT_LOGGER_USER | The basic auth username for authentication to HTTP receiver. |
| EB_EVENT_LOGGER_PASSWORD | The password for the basic auth authentication to HTTP receiver. |

# Output format
The app emits the EventDescription essentially unchanged, except for the addition of a "type" field which I use to help route log entries in [vector](https://vector.dev/).
