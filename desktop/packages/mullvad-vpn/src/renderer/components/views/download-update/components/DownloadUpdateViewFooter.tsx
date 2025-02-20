import React from 'react';
import styled from 'styled-components';

import { messages } from '../../../../../shared/gettext';
import { useAppContext } from '../../../../context';
import { Button, Flex, Icon, LabelTiny, Spinner } from '../../../../lib/components';
import { Progress } from '../../../../lib/components/progress';
import { Colors, Spacings } from '../../../../lib/foundations';
import { DownloadUpdateStatus } from '../../../../redux/download-update/actions';
import { useSelector } from '../../../../redux/store';
import { AnimateHeight } from '../../../AnimateHeight';

const StyledFlex = styled(Flex)`
  background-color: rgba(21, 39, 58, 1);
  position: sticky;
  bottom: 0;
  width: 100%;
`;

const Indicator = styled.div<{ variant?: 'error' | 'warning' }>`
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background-color: ${({ variant }) => (variant === 'error' ? Colors.red : Colors.yellow)};
`;

export const DownloadUpdateViewFooter = () => {
  const progress = useSelector((state) => state.downloadUpdate.progress);
  const status = useSelector((state) => state.downloadUpdate.status);
  const suggestedUpgrade = useSelector((state) => state.version.suggestedUpgrade);
  const { startDownload, stopDownload } = useAppContext();

  const downloadSuggestedUpgrade = React.useCallback(() => {
    if (suggestedUpgrade) {
      // eslint-disable-next-line @typescript-eslint/no-floating-promises
      startDownload(suggestedUpgrade);
    }
  }, [startDownload, suggestedUpgrade]);

  const { text, label } = getDownloadStatusElements(status);

  return (
    <StyledFlex $padding={Spacings.spacing6} $flexDirection="column">
      <AnimateHeight expanded={status !== 'idle'}>
        <Flex
          $flexDirection="column"
          $flex={1}
          $gap={Spacings.spacing5}
          $margin={{ bottom: Spacings.spacing3 }}>
          {label}
          <Progress value={progress}>
            <Progress.Track>
              <Progress.Range />
            </Progress.Track>
            <Progress.Footer>
              <Progress.Percent />
              <Progress.Text>{text}</Progress.Text>
            </Progress.Footer>
          </Progress>
        </Flex>
      </AnimateHeight>
      {status === 'idle' && (
        <Button onClick={downloadSuggestedUpgrade}>
          {
            // TRANSLATORS: Button text to download and install an update
            messages.pgettext('download-update-view', 'Download and install')
          }
        </Button>
      )}
      {status === 'readyForInstall' && (
        <Button>
          {
            // TRANSLATORS: Button text to install an update
            messages.pgettext('download-update-view', 'Install update')
          }
        </Button>
      )}
      {status !== 'idle' && status !== 'readyForInstall' && (
        <Button onClick={stopDownload}>
          {
            // TRANSLATORS: Button text to cancel the download of an update
            messages.pgettext('download-update-view', 'Cancel')
          }
        </Button>
      )}
    </StyledFlex>
  );
};

const texts = {
  starting:
    // TRANSLATORS: Status text displayed below a progress bar when the download of an update is starting
    messages.pgettext('download-update-view', 'Starting download...'),
  downloading:
    // TRANSLATORS: Status text displayed below a progress bar when the update is being downloaded
    messages.pgettext('download-update-view', 'Downloading update...'),
  complete:
    // TRANSLATORS: Status text displayed below a progress bar when the download of an update is complete
    messages.pgettext('download-update-view', 'Download complete!'),
  stopped:
    // TRANSLATORS: Status text displayed below a progress bar when the download of an update has been stopped
    messages.pgettext('download-update-view', 'Download stopped'),
};

const labels = {
  downloading: (
    <Flex $gap={Spacings.spacing3}>
      <LabelTiny>
        {/* TODO: Add correct download URL */}
        {messages.pgettext('download-update-view', 'Downloading from: cdn.mullvad.thirdparty.net')}
      </LabelTiny>
    </Flex>
  ),
  verifying: (
    <Flex $gap={Spacings.spacing3}>
      <Spinner size="small" />
      <LabelTiny>
        {
          // TRANSLATORS: Label displayed above a progress bar when the update is being verified
          messages.pgettext('download-update-view', 'Verifying installer...')
        }
      </LabelTiny>
    </Flex>
  ),
  readyForInstall: (
    <Flex $gap={Spacings.spacing3}>
      <Icon icon="checkmark" color={Colors.green} size="small" />
      <LabelTiny>
        {
          // TRANSLATORS: Label displayed above a progress bar when the update is ready to be installed
          messages.pgettext('download-update-view', 'Verification successful! Ready to install')
        }
      </LabelTiny>
    </Flex>
  ),
  error: (
    <Flex $gap={Spacings.spacing3}>
      <Indicator variant="error" />
      <LabelTiny>
        {
          // TRANSLATORS: Label displayed above a progress bar when an error occurred
          messages.pgettext('download-update-view', 'An error occurred')
        }
      </LabelTiny>
    </Flex>
  ),
};

const getDownloadStatusElements = (status: DownloadUpdateStatus) => {
  switch (status) {
    case 'starting':
      return {
        text: texts.starting,
        label: labels.downloading,
      };
    case 'downloading':
      return {
        text: texts.downloading,
        label: labels.downloading,
      };
    case 'verifying':
      return {
        text: texts.complete,
        label: labels.verifying,
      };
    case 'readyForInstall':
      return {
        text: texts.complete,
        label: labels.readyForInstall,
      };
    case 'error':
      return {
        text: texts.stopped,
        label: labels.error,
      };
    default:
      return {
        text: '',
        label: <></>,
      };
  }
};
