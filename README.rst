otp-cop
=======

  At this point, these services all natively have the ability to mandate 2FA, therefore this project is retired.

Verify that everyone in your organization has 2fa set up. Works with the
following services:

* Slack
* Github

To use:

.. code-block:: console

    $ git clone https://github.com/alex/otp-cop
    $ cd otp-cop
    $ cargo build
    $ ./target/debug/otp-cop

Now you go and yell at all these people.

Services
--------

Slack
+++++

.. code-block:: console

    $ otp-cop --slack-token='<token>'

You can obtain a token online: https://api.slack.com/web#authentication

Github
++++++

.. code-block:: console

    $ otp-cop --github-org='<org>' --github-username='<username>' --github-password='<password>'

You can generate a Github personal access token online:
https://help.github.com/articles/creating-an-access-token-for-command-line-use/.

The user needs to be an owner of the organization.

``otp-cop`` requires the ``read:org`` scope.

Github Enterprise users must specify the API endpoint URL.

.. code-block:: console

    $ otp-cop --github-endpoint='<endpoint>' --github-org='<org>' --github-username='<username>' --github-password='<password>'
