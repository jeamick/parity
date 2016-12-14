// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

import { observer } from 'mobx-react';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';

import { Button } from '~/ui';
import { CancelIcon, DoneIcon, NextIcon, SnoozeIcon } from '~/ui/Icons';
import Modal, { Busy, Completed } from '~/ui/Modal';

import ModalStore, { STEP_COMPLETED, STEP_ERROR, STEP_INFO, STEP_UPDATING } from './modalStore';
import UpgradeStore from './upgradeStore';
import styles from './upgradeParity.css';

@observer
export default class UpgradeParity extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  store = new ModalStore(new UpgradeStore(this.context.api));

  render () {
    if (!this.store.upgrade.available || !this.store.showUpgrade) {
      return null;
    }

    return (
      <Modal
        actions={ this.renderActions() }
        current={ this.store.step }
        steps={ [
          <FormattedMessage
            id='upgradeParity.step.info'
            defaultMessage='upgrade available' />,
          <FormattedMessage
            id='upgradeParity.step.updating'
            defaultMessage='upgrading parity' />,
          this.store.step === STEP_ERROR
            ? <FormattedMessage
              id='upgradeParity.step.completed'
              defaultMessage='upgrade completed' />
            : <FormattedMessage
              id='upgradeParity.step.error'
              defaultMessage='error' />
        ] }
        visible>
        { this.renderStep() }
      </Modal>
    );
  }

  renderActions () {
    const closeButton =
      <Button
        icon={ <CancelIcon /> }
        label={
          <FormattedMessage
            id='upgradeParity.button.close'
            defaultMessage='close' />
        }
        onClick={ this.store.closeModal } />;
    const doneButton =
      <Button
        icon={ <DoneIcon /> }
        label={
          <FormattedMessage
            id='upgradeParity.button.done'
            defaultMessage='done' />
        }
        onClick={ this.store.closeModal } />;

    switch (this.store.step) {
      case STEP_INFO:
        return [
          <Button
            icon={ <SnoozeIcon /> }
            label={
              <FormattedMessage
                id='upgradeParity.button.snooze'
                defaultMessage='ask me tomorrow' />
            }
            onClick={ this.store.snoozeTillTomorrow } />,
          <Button
            icon={ <NextIcon /> }
            label={
              <FormattedMessage
                id='upgradeParity.button.upgrade'
                defaultMessage='upgrade now' />
            }
            onClick={ this.store.upgradeNow } />,
          closeButton
        ];

      case STEP_UPDATING:
        return [
          closeButton
        ];

      case STEP_COMPLETED:
      case STEP_ERROR:
        return [
          doneButton
        ];
    }
  }

  renderStep () {
    const { available, consensusCapability, error, upgrading, version } = this.store.upgrade;

    const currentversion = this.renderVersion(version);
    const newversion = upgrading
      ? this.renderVersion(upgrading.version)
      : this.renderVersion(available.version);

    switch (this.store.step) {
      case STEP_INFO:
        let consensusInfo = null;
        if (consensusCapability === 'capable') {
          consensusInfo = (
            <div>
              <FormattedMessage
                id='upgradeParity.consensus.capable'
                defaultMessage='Your current Parity version is capable of handling the nework requirements.' />
            </div>
          );
        } else if (consensusCapability.capableUntil) {
          consensusInfo = (
            <div>
              <FormattedMessage
                id='upgradeParity.consensus.capableUntil'
                defaultMessage='Your current Parity version is capable of handling the nework requirements until block {blockNumber}'
                values={ {
                  blockNumber: consensusCapability.capableUntil
                } } />
            </div>
          );
        } else if (consensusCapability.incapableSince) {
          consensusInfo = (
            <div>
              <FormattedMessage
                id='upgradeParity.consensus.incapableSince'
                defaultMessage='Your current Parity version is incapable of handling the nework requirements since block {blockNumber}'
                values={ {
                  blockNumber: consensusCapability.incapableSince
                } } />
            </div>
          );
        }

        return (
          <div className={ styles.infoStep }>
            <div>
              <FormattedMessage
                id='upgradeParity.info.upgrade'
                defaultMessage='A new version of Parity, version {newversion} is available as an upgrade from your current version {currentversion}'
                values={ {
                  currentversion,
                  newversion
                } } />
            </div>
            { consensusInfo }
          </div>
        );

      case STEP_UPDATING:
        return (
          <Busy
            title={
              <FormattedMessage
                id='upgradeParity.busy'
                defaultMessage='Your upgrade to Parity {newversion} is currently in progress'
                values={ {
                  newversion
                } } />
            } />
        );

      case STEP_COMPLETED:
        return (
          <Completed>
            <FormattedMessage
              id='upgradeParity.completed'
              defaultMessage='Your upgrade to Parity {newversion} has been successfully completed.'
              values={ {
                newversion
              } } />
          </Completed>
        );

      case STEP_ERROR:
        return (
          <Completed>
            <div>
              <FormattedMessage
                id='upgradeParity.failed'
                defaultMessage='Your upgrade to Parity {newversion} has failed with an error.'
                values={ {
                  newversion
                } } />
            </div>
            <div className={ styles.error }>
              { error.message }
            </div>
          </Completed>
        );
    }
  }

  renderVersion (versionInfo) {
    const { track, version } = versionInfo;

    return `${version.major}.${version.minor}.${version.patch}-${track}`;
  }
}