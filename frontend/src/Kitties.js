import React, { useEffect, useState } from 'react'
import { Form, Grid } from 'semantic-ui-react'

import { useSubstrate } from './substrate-lib'
import { TxButton } from './substrate-lib/components'

import KittyCards from './KittyCards'

export default function Kitties (props) {
  const { api, keyring } = useSubstrate()
  const { accountPair } = props

  const [kitties, setKitties] = useState([])
  const [status, setStatus] = useState('')

  const [kittyCount, setKittyCount] = useState(0)
  const [kittyOwners, setKittyOwners] = useState([])
  const [kittiesDNA, setKittiesDNA] = useState([])

  const fetchKitties = () => {
    let _kittyCount

    api.query.kittiesModule.kittiesCount(i => {
      if (i.isSome) {
        _kittyCount = i.unwrap().toNumber()
      }
      setKittyCount(_kittyCount)

      const _kittiesDNA = []
      const _kittyIndexs = [...Array(_kittyCount).keys()]
      api.query.kittiesModule.kitties.multi(_kittyIndexs, (_kitties) => {
        for (const i in _kitties) {
          if (_kitties[i].isSome) {
            const _kitty = _kitties[i].unwrap().toU8a()
            _kittiesDNA.push(_kitty)
          }
        }
        setKittiesDNA(_kittiesDNA)
      })

      const _kittyOwners = []
      api.query.kittiesModule.kittyOwners.multi(_kittyIndexs, (_owners) => {
        for (const i in _owners) {
          if (_owners[i].isSome) {
            const _owner = _owners[i].unwrap().toString()
            _kittyOwners.push(_owner)
          }
        }
        setKittyOwners(_kittyOwners)
      })
    })
  }

  const populateKitties = () => {
    const _kitties = []

    // I don't know why the wrong data sometimes appears. I so suck at js
    if (kittiesDNA.length > kittyCount || kittyOwners.length > kittyCount) {
      return
    }

    for (let i = 0; i < kittyCount; i++) {
      _kitties.push({
        id: i,
        dna: kittiesDNA[i],
        owner: kittyOwners[i]
      })
    }

    setKitties(_kitties)
  }

  useEffect(fetchKitties, [api, keyring,status])
  useEffect(populateKitties, [kittyCount, kittyOwners, kittiesDNA, status])

  return <Grid.Column width={16}>
    <h1>猫咪</h1>
    <KittyCards kitties={kitties} accountPair={accountPair} setStatus={setStatus}/>
    <Form style={{ margin: '1em 0' }}>
      <Form.Field style={{ textAlign: 'center' }}>
        <TxButton
          accountPair={accountPair} label='创建猫咪' type='SIGNED-TX' setStatus={setStatus}
          attrs={{
            palletRpc: 'kittiesModule',
            callable: 'create',
            inputParams: [],
            paramFields: []
          }}
        />
      </Form.Field>
    </Form>
    <div style={{ overflowWrap: 'break-word' }}>{status}</div>
  </Grid.Column>
}
