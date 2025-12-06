import React from 'react';
import { defineMessages, injectIntl, FormattedMessage } from 'react-intl';

const messages = defineMessages({
    placeholder: { id: 'compose_form.placeholder', defaultMessage: 'What is on your mind?' },
    publish: { id: 'compose_form.publish', defaultMessage: 'Post' },
});

class ComposeForm extends React.PureComponent {
    render() {
        const { intl } = this.props;

        return (
            <div className="compose-form">
                <textarea
                    placeholder={intl.formatMessage(messages.placeholder)}
                />
                <div className="compose-form__publish">
                    <Button
                        text={intl.formatMessage(messages.publish)}
                        onClick={this.handleSubmit}
                    />
                </div>
                <div className="privacy-note">
                    <FormattedMessage id="compose_form.privacy_disclaimer" defaultMessage="Your private post will be delivered to mentioned users only." />
                </div>
            </div>
        );
    }
}

export default injectIntl(ComposeForm);
