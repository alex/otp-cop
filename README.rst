otp-cop
=======

Verify that everyone in your organization has 2fa set up. Works with the
following services:

* Slack
* Github

To use:

.. code-block:: console

    $ git clone https://github.com/alex/otp-cop
    $ cd otp-cop
    $ cargo build
    $ ./target/debug/otp-cop \
        --slack-token='<token>' \
        --github-org='<org>' --github-username='<username>' --github-password='<password>'

Now you go and yell at all these people.
