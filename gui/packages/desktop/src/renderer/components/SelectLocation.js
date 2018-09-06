// @flow

import * as React from 'react';
import ReactDOM from 'react-dom';
import { View } from 'reactxp';
import { Accordion } from '@mullvad/components';
import { Layout, Container } from './Layout';
import CustomScrollbars from './CustomScrollbars';
import NavigationBar, { CloseBarItem } from './NavigationBar';
import SettingsHeader, { HeaderTitle, HeaderSubTitle } from './SettingsHeader';
import * as Cell from './Cell';
import styles from './SelectLocationStyles';

import type {
  SettingsReduxState,
  RelayLocationRedux,
  RelayLocationCityRedux,
  RelayLocationRelayRedux,
} from '../redux/settings/reducers';
import type { RelayLocation } from '../lib/daemon-rpc';

export type SelectLocationProps = {
  settings: SettingsReduxState,
  onClose: () => void,
  onSelect: (location: RelayLocation) => void,
};

type State = {
  expanded: Array<string>,
};

export default class SelectLocation extends React.Component<SelectLocationProps, State> {
  _selectedCell: ?Cell.CellButton;
  _scrollView: ?CustomScrollbars;

  state = {
    expanded: [],
  };

  constructor(props: SelectLocationProps, context?: any) {
    super(props, context);

    // set initially expanded country based on relaySettings
    const relaySettings = this.props.settings.relaySettings;
    if (relaySettings.normal) {
      const { location } = relaySettings.normal;
      if (location === 'any') {
        // no-op
      } else if (location.country) {
        this.state.expanded.push(location.country);
      } else if (location.city) {
        const countryCode = location.city[0];

        this.state.expanded.push(countryCode);
      } else if (location.hostname) {
        const countryCode = location.hostname[0];
        const cityCode = location.hostname[1];

        this.state.expanded.push(countryCode);
        this.state.expanded.push(`${countryCode}_${cityCode}`);
      }
    }
  }

  componentDidMount() {
    // restore scroll to selected cell
    const cell = this._selectedCell;
    const scrollView = this._scrollView;

    if (scrollView && cell) {
      // eslint-disable-next-line react/no-find-dom-node
      const cellDOMNode = ReactDOM.findDOMNode(cell);

      if (cellDOMNode instanceof HTMLElement) {
        scrollView.scrollToElement(cellDOMNode, 'middle');
      }
    }
  }

  render() {
    return (
      <Layout>
        <Container>
          <View style={styles.select_location}>
            <NavigationBar>
              <CloseBarItem action={this.props.onClose} />
            </NavigationBar>
            <View style={styles.container}>
              <SettingsHeader style={styles.title_header}>
                <HeaderTitle>Select location</HeaderTitle>
              </SettingsHeader>

              <CustomScrollbars autoHide={true} ref={(ref) => (this._scrollView = ref)}>
                <View style={styles.content}>
                  <SettingsHeader style={styles.subtitle_header}>
                    <HeaderSubTitle>
                      While connected, your real location is masked with a private and secure
                      location in the selected region
                    </HeaderSubTitle>
                  </SettingsHeader>

                  {this.props.settings.relayLocations.map((relayCountry) => {
                    return this._renderCountry(relayCountry);
                  })}
                </View>
              </CustomScrollbars>
            </View>
          </View>
        </Container>
      </Layout>
    );
  }

  _isSelected(selectedLocation: RelayLocation) {
    const { relaySettings } = this.props.settings;
    if (relaySettings.normal) {
      const otherLocation = relaySettings.normal.location;

      if (
        selectedLocation.country &&
        otherLocation.country &&
        selectedLocation.country === otherLocation.country
      ) {
        return true;
      }

      if (Array.isArray(selectedLocation.city) && Array.isArray(otherLocation.city)) {
        const selectedCity = selectedLocation.city;
        const otherCity = otherLocation.city;

        return (
          selectedCity.length === otherCity.length &&
          selectedCity.every((v, i) => v === otherCity[i])
        );
      }

      if (Array.isArray(selectedLocation.hostname) && Array.isArray(otherLocation.hostname)) {
        const selectedRelay = selectedLocation.hostname;
        const otherRelay = otherLocation.hostname;

        return (
          selectedRelay.length === otherRelay.length &&
          selectedRelay.every((v, i) => v === otherRelay[i])
        );
      }
    }
    return false;
  }

  _toggleCollapse = (countryCode: string) => {
    this.setState((state) => {
      const expanded = state.expanded.slice();
      const index = expanded.indexOf(countryCode);
      if (index === -1) {
        expanded.push(countryCode);
      } else {
        expanded.splice(index, 1);
      }
      return { expanded };
    });
  };

  _relayStatusIndicator(active: boolean, isSelected: boolean) {
    const statusClass = active ? styles.relay_status__active : styles.relay_status__inactive;

    return isSelected ? (
      <Cell.Img style={styles.tick_icon} source="icon-tick" height={24} width={24} />
    ) : (
      <View style={[styles.relay_status, statusClass]} />
    );
  }

  _renderCountry(relayCountry: RelayLocationRedux) {
    const isSelected = this._isSelected({ country: relayCountry.code });

    const onRef = isSelected
      ? (element) => {
          this._selectedCell = element;
        }
      : undefined;

    // either expanded by user or when the city selected within the country
    const isExpanded = this.state.expanded.includes(relayCountry.code);

    const hasChildren =
      relayCountry.cities.length > 1 ||
      (relayCountry.cities.length == 1 && relayCountry.cities[0].relays.length > 1);

    const handleSelect =
      relayCountry.hasActiveRelays && !isSelected
        ? () => {
            this.props.onSelect({ country: relayCountry.code });
          }
        : undefined;

    const handleCollapse = (e) => {
      this._toggleCollapse(relayCountry.code);
      e.stopPropagation();
    };

    return (
      <View key={relayCountry.code} style={styles.country}>
        <Cell.CellButton
          cellHoverStyle={isSelected ? styles.cell_selected : null}
          style={isSelected ? styles.cell_selected : styles.cell}
          onPress={handleSelect}
          disabled={!relayCountry.hasActiveRelays}
          testName="country"
          ref={onRef}>
          {this._relayStatusIndicator(relayCountry.hasActiveRelays, isSelected)}

          <Cell.Label>{relayCountry.name}</Cell.Label>

          {hasChildren ? (
            <Cell.Img
              style={styles.collapse_button}
              hoverStyle={styles.expand_chevron_hover}
              onPress={handleCollapse}
              source={isExpanded ? 'icon-chevron-up' : 'icon-chevron-down'}
              height={24}
              width={24}
            />
          ) : null}
        </Cell.CellButton>

        {hasChildren && (
          <Accordion height={isExpanded ? 'auto' : 0}>
            {relayCountry.cities.map((relayCity) => this._renderCity(relayCountry.code, relayCity))}
          </Accordion>
        )}
      </View>
    );
  }

  _renderCity(countryCode: string, relayCity: RelayLocationCityRedux) {
    const expandedCode = `${countryCode}_${relayCity.code}`;
    const relayLocation: RelayLocation = { city: [countryCode, relayCity.code] };

    const isSelected = this._isSelected(relayLocation);

    const onRef = isSelected
      ? (element) => {
          this._selectedCell = element;
        }
      : undefined;

    // either expanded by user or when the city or a relay from the city is selected
    const isExpanded = this.state.expanded.includes(expandedCode);

    const handleSelect =
      relayCity.hasActiveRelays && !isSelected
        ? () => {
            this.props.onSelect(relayLocation);
          }
        : undefined;

    const handleCollapse = (e) => {
      this._toggleCollapse(expandedCode);
      e.stopPropagation();
    };

    return (
      <View key={expandedCode}>
        <Cell.CellButton
          onPress={handleSelect}
          disabled={!relayCity.hasActiveRelays}
          cellHoverStyle={isSelected ? styles.sub_cell__selected : null}
          style={isSelected ? styles.sub_cell__selected : styles.sub_cell}
          testName="city"
          ref={onRef}>
          {this._relayStatusIndicator(relayCity.hasActiveRelays, isSelected)}

          <Cell.Label>{relayCity.name}</Cell.Label>

          {relayCity.relays.length > 1 ? (
            <Cell.Img
              style={styles.collapse_button}
              hoverStyle={styles.expand_chevron_hover}
              onPress={handleCollapse}
              source={isExpanded ? 'icon-chevron-up' : 'icon-chevron-down'}
              height={24}
              width={24}
            />
          ) : null}
        </Cell.CellButton>

        {relayCity.relays.length > 1 && (
          <Accordion height={isExpanded ? 'auto' : 0}>
            {relayCity.relays.map((relay) => this._renderRelay(countryCode, relayCity.code, relay))}
          </Accordion>
        )}
      </View>
    );
  }

  _renderRelay(countryCode: string, cityCode: string, relay: RelayLocationRelayRedux) {
    const relayLocation: RelayLocation = { hostname: [countryCode, cityCode, relay.hostname] };

    const isSelected = this._isSelected(relayLocation);

    const onRef = isSelected
      ? (element) => {
          this._selectedCell = element;
        }
      : undefined;

    const handleSelect = !isSelected
      ? () => {
          this.props.onSelect(relayLocation);
        }
      : undefined;

    return (
      <Cell.CellButton
        key={`${countryCode}_${cityCode}_${relay.hostname}`}
        onPress={handleSelect}
        cellHoverStyle={isSelected ? styles.sub_sub_cell__selected : null}
        style={isSelected ? styles.sub_sub_cell__selected : styles.sub_sub_cell}
        testName="relay"
        ref={onRef}>
        {this._relayStatusIndicator(true, isSelected)}

        <Cell.Label>{relay.hostname}</Cell.Label>
      </Cell.CellButton>
    );
  }
}
